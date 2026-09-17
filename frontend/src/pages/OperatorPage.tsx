import { useEffect, useState } from "react";
import { Link } from "react-router-dom";

type Settings = {
  display_code: string;
  privacy_mode: string;
  sound_enabled: boolean;
};

export default function OperatorPage() {
  const [token, setToken] = useState("");
  const [settings, setSettings] = useState<Settings | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const stored = window.localStorage.getItem("token");
    if (stored) {
      setToken(stored);
    }
  }, []);

  async function loadSettings() {
    setError(null);
    try {
      const response = await fetch("/v2/queue/settings", {
        headers: { Authorization: `Bearer ${token}` },
      });
      const body = await response.json();
      if (!response.ok || !body.success) {
        throw new Error(body.message || "Falha ao carregar");
      }
      setSettings(body.data);
    } catch {
      setError("Não foi possível carregar o código da TV. Confirme o login.");
    }
  }

  return (
    <main className="operator">
      <h1>Painel de chamadas</h1>
      <p>
        A recepção marca <strong>Paciente chegou</strong> na agenda. O médico
        chama no prontuário. Esta tela só configura a TV.
      </p>
      <label>
        Token de sessão
        <input
          value={token}
          onChange={(e) => setToken(e.target.value)}
          aria-label="Token de sessão"
        />
      </label>
      <button type="button" onClick={() => void loadSettings()}>
        Carregar código da TV
      </button>
      {error ? <p role="alert">{error}</p> : null}
      {settings ? (
        <section>
          <p>
            Código: <strong>{settings.display_code}</strong>
          </p>
          <p>
            Abra na TV:{" "}
            <Link to={`/display/${settings.display_code}`}>
              /display/{settings.display_code}
            </Link>
          </p>
          <p>Privacidade: {settings.privacy_mode}</p>
        </section>
      ) : null}
    </main>
  );
}
