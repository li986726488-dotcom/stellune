import type { AppErrorShape } from "@/types";

export function toAppError(error: unknown): AppErrorShape {
  if (
    typeof error === "object" &&
    error !== null &&
    "code" in error &&
    "message" in error
  ) {
    const candidate = error as Partial<AppErrorShape>;
    return {
      code: String(candidate.code),
      message: String(candidate.message),
      retryable: Boolean(candidate.retryable),
    };
  }

  return {
    code: "INTERNAL_ERROR",
    message: error instanceof Error ? error.message : "暂时无法连接宇宙，请稍后再试。",
    retryable: true,
  };
}
