import { Injectable, NgZone } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn, Event } from '@tauri-apps/api/event';

@Injectable({
  providedIn: 'root',
})
export class TauriService {
  constructor(private zone: NgZone) {}

  /**
   * Invokes a command on the Rust backend.
   * @param command The name of the command.
   * @param args Optional arguments for the command.
   */
  async call<T>(command: string, args?: any): Promise<T> {
    try {
      return await invoke<T>(command, args);
    } catch (error) {
      console.error(`Tauri command "${command}" failed:`, error);
      throw error;
    }
  }

  /**
   * Listens for an event from the Rust backend.
   * Automatically handles NgZone wrapping so UI updates work correctly.
   * @param event The event name.
   * @param handler The callback function.
   * @returns A promise that resolves to an unlisten function.
   */
  async on<T>(event: string, handler: (payload: T) => void): Promise<UnlistenFn> {
    return await listen<T>(event, (eventObj: Event<T>) => {
      this.zone.run(() => {
        handler(eventObj.payload);
      });
    });
  }
}
