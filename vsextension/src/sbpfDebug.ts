import * as vscode from 'vscode';
import {
  DebugSession,
  InitializedEvent,
  OutputEvent,
  Scope,
  TerminatedEvent,
  StoppedEvent
} from 'vscode-debugadapter';
import { DebugProtocol } from '@vscode/debugprotocol';
import * as heliosVM from '../wasm/helios_vm';

interface LaunchRequestArguments extends DebugProtocol.LaunchRequestArguments {
  accountNumber?: number;
  instructionData?: number[] | string;
}

class SBPFDebugSession extends DebugSession {
  private _registersVarRef = 1000;
  private _rodataVarRef = 2000;
  private _memoryVarRef = 3000;
  private _currentRegisters: { name: string, 
                              value: string, 
                              type: "int" | "addr" | "null" }[] = [];
  //
  private _currentRodata: { label: string, 
                            address: number, 
                            value: string }[] = [];
  //
  private _currentMemory: { label: string, 
                            value: string }[] = [];

  private _frameId = 1;

  constructor() {
    super();
  }

  protected initializeRequest(
    response: DebugProtocol.InitializeResponse,
    args: DebugProtocol.InitializeRequestArguments
  ): void {
    console.log('Debug session initializing...', args);
    response.body = response.body || {};
    response.body.supportsConfigurationDoneRequest = true;
    response.body.supportsEvaluateForHovers = true;
    response.body.supportsStepInTargetsRequest = true;
    this.sendResponse(response);
    this.sendEvent(new InitializedEvent());
  }

  protected scopesRequest(
    response: DebugProtocol.ScopesResponse,
    args: DebugProtocol.ScopesArguments
  ): void {
    const scopes = [
      new Scope("Registers", this._registersVarRef, false),
      new Scope("Memory", this._memoryVarRef, false)
    ];

    // Only add rodata scope if it exists
    if (this._currentRodata.length > 0) {
      scopes.push(new Scope(".rodata", this._rodataVarRef, false));
    }

    response.body = { scopes };
    this.sendResponse(response);
  }

  protected stackTraceRequest(
    response: DebugProtocol.StackTraceResponse,
    args: DebugProtocol.StackTraceArguments
  ): void {
    response.body = {
      stackFrames: [{
        id: this._frameId,
        name: "e",
        line: heliosVM.get_line_number(),
        column: 1,
        source: {
          name: "program.sbpf",
          path: vscode.window.activeTextEditor?.document.uri.fsPath
        }
      }],
      totalFrames: 1
    };
    this.sendResponse(response);
  }

  protected variablesRequest(response: DebugProtocol.VariablesResponse, args: DebugProtocol.VariablesArguments): void {
    if (args.variablesReference === this._registersVarRef) {
      response.body = {
            variables: this._currentRegisters.map(r => ({
                name: r.name,
                value: r.value,
                type: r.type,
                variablesReference: 0
            }))
        };
    } else if (args.variablesReference === this._rodataVarRef) {
        response.body = {
            variables: this._currentRodata.map(r => ({
                name: `${r.label} @ 0x${r.address.toString(16)}`,
                value: r.value,
                variablesReference: 0
            }))
        };
    } else if (args.variablesReference === this._memoryVarRef) {
        response.body = {
            variables: this._currentMemory.map(m => ({
                name: m.label,
                value: m.value,
                variablesReference: 0
            }))
        };
    } else {
        response.body = { variables: [] };
    }
    this.sendResponse(response);
  }

  protected nextRequest(
    response: DebugProtocol.NextResponse,
    args: DebugProtocol.NextArguments
  ): void {
    // Execute one instruction
    heliosVM.step();

    // Update state after execution
    this._currentRegisters = heliosVM.get_registers();
    this._currentRodata = heliosVM.get_rodata();
    this._currentMemory = heliosVM.get_memory();

    if (heliosVM.is_exited()) {
      this.sendEvent(new TerminatedEvent());
      this.sendEvent(new OutputEvent(`Program output: ${heliosVM.get_log()}\n`));
      this.sendResponse(response);
      return;
    }

    const editor = vscode.window.activeTextEditor;
    // Tell VS Code we've stopped after the step
    const stoppedEvent = new StoppedEvent('breakpoint', 1);  // Use thread ID 1
    (stoppedEvent as any).body = {
      reason: 'breakpoint',
      threadId: 1,
      source: {
        name: 'program.sbpf',
        path: editor?.document.uri.fsPath
      },
      line: heliosVM.get_line_number(),
      column: 1
    };

    if (heliosVM.get_log() !== '') {
      this.sendEvent(new OutputEvent(`${heliosVM.get_log()}\n`));
      heliosVM.clear_log();
    }

    this.sendEvent(stoppedEvent);
    this.sendResponse(response);
  }
  
  protected launchRequest(
    response: DebugProtocol.LaunchResponse,
    args: DebugProtocol.LaunchRequestArguments
  ): void {
    const editor = vscode.window.activeTextEditor;
    const code = editor?.document.getText() || '';

    heliosVM.clear_log();
    heliosVM.initialize(code, editor?.document.uri.fsPath || '');
    
    const { accountNumber = 0, instructionData = [] } = args as LaunchRequestArguments;
    
    let instructionBytes: Uint8Array;
    let instructionDataType: 'string' | 'number';
    if (typeof instructionData === 'string') {
      // Convert string to Uint8Array (treating each character as a byte)
      instructionBytes = new Uint8Array(instructionData.split('').map(char => char.charCodeAt(0)));
      instructionDataType = 'string';
    } else {
      // instructionData is already a number array
      instructionBytes = new Uint8Array(instructionData);
      instructionDataType = 'number';
    }
    
    heliosVM.load_input_data(BigInt(accountNumber), instructionBytes, instructionDataType);

    // Get initial state
    this._currentRegisters = heliosVM.get_registers();
    this._currentRodata = heliosVM.get_rodata();
    this._currentMemory = heliosVM.get_memory();

    // Tell VS Code the program is ready to be debugged
    const stoppedEvent = new StoppedEvent('breakpoint', 1);  // Use thread ID 1
    (stoppedEvent as any).body = {
      reason: 'breakpoint',
      threadId: 1,
      source: {
        name: 'program.sbpf',
        path: editor?.document.uri.fsPath
      },
      line: heliosVM.get_line_number(),
      column: 1
    };
    this.sendEvent(stoppedEvent);

    this.sendResponse(response);
  }

  protected disconnectRequest(
    response: DebugProtocol.DisconnectResponse,
    args: DebugProtocol.DisconnectArguments
  ): void {
    this.sendEvent(new TerminatedEvent());
    this.sendResponse(response);
  }

  protected configurationDoneRequest(
    response: DebugProtocol.ConfigurationDoneResponse,
    args: DebugProtocol.ConfigurationDoneArguments
  ): void {
    this.sendResponse(response);
  }

  protected threadsRequest(response: DebugProtocol.ThreadsResponse): void {
    response.body = {
      threads: [
        {
          id: 1,
          name: "Main Thread"
        }
      ]
    };
    this.sendResponse(response);
  }
}

// Factory to run the adapter inline (in the extension host)
export class InlineDebugAdapterFactory
  implements vscode.DebugAdapterDescriptorFactory
{
  createDebugAdapterDescriptor(
    _session: vscode.DebugSession,
    _executable: vscode.DebugAdapterExecutable | undefined
  ): vscode.ProviderResult<vscode.DebugAdapterDescriptor> {
    const session = new SBPFDebugSession();
    return new vscode.DebugAdapterInlineImplementation(session);
  }

  dispose() {}
}
