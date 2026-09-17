export type DisplayCall = {
  display_name: string;
  room_name?: string | null;
  employee_name?: string | null;
  called_at: string;
};

export type DisplayPayload = {
  current: DisplayCall | null;
  recent: DisplayCall[];
  sound_enabled: boolean;
  privacy_mode: string;
};

export async function fetchDisplay(
  code: string,
  fetcher: typeof fetch = fetch,
): Promise<DisplayPayload> {
  const response = await fetcher(`/v2/queue/display/${code}`);
  if (!response.ok) {
    throw new Error("Painel indisponível");
  }
  const body = (await response.json()) as {
    success?: boolean;
    data?: DisplayPayload;
  };
  if (!body.success || !body.data) {
    throw new Error("Painel indisponível");
  }
  return body.data;
}
