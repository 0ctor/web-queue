export type DisplayNameMode = "full" | "first_last_initial" | "initials";

export function formatDisplayName(
  fullName: string,
  mode: DisplayNameMode,
): string {
  const parts = fullName.trim().split(/\s+/).filter(Boolean);
  if (parts.length === 0) {
    return "Paciente";
  }
  if (mode === "full") {
    return parts.join(" ");
  }
  const first = parts[0];
  const last = parts.length > 1 ? parts[parts.length - 1] : "";
  if (mode === "initials") {
    const firstInitial = `${first.charAt(0).toUpperCase()}.`;
    if (!last) return firstInitial;
    return `${firstInitial} ${last.charAt(0).toUpperCase()}.`;
  }
  if (!last) return first;
  return `${first} ${last.charAt(0).toUpperCase()}.`;
}

export function normalizeDisplayCode(raw: string): string {
  return raw
    .toUpperCase()
    .replace(/[^A-Z0-9]/g, "")
    .slice(0, 6);
}
