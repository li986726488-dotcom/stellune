import type { CommandClient } from "@/services/command-client";

let commandClient: CommandClient | null = null;

export async function initializeCommandClient(): Promise<void> {
  if (commandClient) return;

  if (import.meta.env.MODE === "e2e") {
    const { MockCommandClient } = await import(
      "../../tests/e2e/mocks/command-client"
    );
    commandClient = new MockCommandClient();
    return;
  }

  const { TauriCommandClient } = await import(
    "@/services/tauri-command-client"
  );
  commandClient = new TauriCommandClient();
}

export function useCommandClient(): CommandClient {
  if (!commandClient) {
    throw new Error("CommandClient has not been initialized.");
  }
  return commandClient;
}
