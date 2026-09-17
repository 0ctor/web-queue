import { describe, expect, it, vi } from "vitest";
import { fetchDisplay } from "./displayApi";

describe("fetchDisplay", () => {
  it("lê a última chamada do painel público", async () => {
    const fetcher = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        success: true,
        data: {
          current: {
            display_name: "Maria S.",
            room_name: "Consultório 1",
            employee_name: "Dr. João",
            called_at: "2026-09-17T12:00:00Z",
          },
          recent: [],
          sound_enabled: true,
          privacy_mode: "first_last_initial",
        },
      }),
    });
    const data = await fetchDisplay("ABC123", fetcher as unknown as typeof fetch);
    expect(data.current?.display_name).toBe("Maria S.");
    expect(fetcher).toHaveBeenCalledWith("/v2/queue/display/ABC123");
  });
});
