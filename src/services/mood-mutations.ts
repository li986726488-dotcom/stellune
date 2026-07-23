import { useCommandClient } from "@/services/commands";
import type { Mood } from "@/types";

let mutationQueue: Promise<void> = Promise.resolve();

export function enqueueMoodMutation<T>(
  mutation: () => Promise<T>,
): Promise<T> {
  const result = mutationQueue.then(mutation, mutation);
  mutationQueue = result.then(
    () => undefined,
    () => undefined,
  );
  return result;
}

export function saveMoodSerialized(mood: Mood) {
  return enqueueMoodMutation(() => useCommandClient().saveMood(mood));
}
