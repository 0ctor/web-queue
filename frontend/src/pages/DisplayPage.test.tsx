import { render, screen } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { describe, expect, it, vi, beforeEach } from "vitest";
import DisplayPage from "./DisplayPage";

vi.mock("../lib/displayApi", () => ({
  fetchDisplay: vi.fn().mockResolvedValue({
    current: {
      display_name: "Maria S.",
      room_name: "Consultório 1",
      employee_name: "Dr. João",
      called_at: "2026-09-17T12:00:00Z",
    },
    recent: [{ display_name: "Ana P.", called_at: "2026-09-17T11:50:00Z" }],
    sound_enabled: false,
    privacy_mode: "first_last_initial",
  }),
}));

describe("DisplayPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("exibe o paciente chamado na TV", async () => {
    render(
      <MemoryRouter initialEntries={["/display/ABC123"]}>
        <Routes>
          <Route path="/display/:code" element={<DisplayPage />} />
        </Routes>
      </MemoryRouter>,
    );
    expect(await screen.findByText("Maria S.")).toBeInTheDocument();
    expect(screen.getByText(/Consultório 1/)).toBeInTheDocument();
  });
});
