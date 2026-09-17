import { useEffect, useRef, useState } from "react";
import { useParams } from "react-router-dom";
import { fetchDisplay, type DisplayPayload } from "../lib/displayApi";
import { normalizeDisplayCode } from "../lib/displayName";

function beep() {
  try {
    const ctx = new AudioContext();
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.type = "sine";
    osc.frequency.value = 880;
    gain.gain.value = 0.08;
    osc.connect(gain);
    gain.connect(ctx.destination);
    osc.start();
    osc.stop(ctx.currentTime + 0.35);
  } catch {
    // TV sem Web Audio
  }
}

export default function DisplayPage() {
  const { code = "" } = useParams();
  const displayCode = normalizeDisplayCode(code);
  const [data, setData] = useState<DisplayPayload | null>(null);
  const [error, setError] = useState<string | null>(null);
  const lastCallAt = useRef<string | null>(null);

  useEffect(() => {
    if (displayCode.length !== 6) {
      setError("Código do painel inválido.");
      return;
    }
    let cancelled = false;
    const load = async () => {
      try {
        const next = await fetchDisplay(displayCode);
        if (cancelled) return;
        setData(next);
        setError(null);
        const stamp = next.current?.called_at ?? null;
        if (stamp && stamp !== lastCallAt.current) {
          if (lastCallAt.current !== null && next.sound_enabled) {
            beep();
          }
          lastCallAt.current = stamp;
        }
      } catch {
        if (!cancelled) {
          setError("Não foi possível atualizar o painel.");
        }
      }
    };
    void load();
    const id = window.setInterval(() => void load(), 2000);
    return () => {
      cancelled = true;
      window.clearInterval(id);
    };
  }, [displayCode]);

  if (error && !data) {
    return (
      <main className="screen">
        <p>{error}</p>
      </main>
    );
  }

  return (
    <main className="screen">
      <header>
        <p className="brand">Octor · Sala de espera</p>
      </header>
      {data?.current ? (
        <section className="current" aria-live="polite">
          <p className="label">Por favor, dirija-se ao atendimento</p>
          <h1>{data.current.display_name}</h1>
          <p className="meta">
            {[data.current.employee_name, data.current.room_name]
              .filter(Boolean)
              .join(" · ")}
          </p>
        </section>
      ) : (
        <section className="current empty">
          <h1>Aguardando chamada</h1>
          <p className="meta">O próximo paciente será anunciado aqui.</p>
        </section>
      )}
      {data?.recent?.length ? (
        <ul className="recent">
          {data.recent.map((item) => (
            <li key={`${item.display_name}-${item.called_at}`}>
              {item.display_name}
            </li>
          ))}
        </ul>
      ) : null}
    </main>
  );
}
