/**
 * Utilitário de cópia para a área de transferência com suporte a fallback e feedback temporizado.
 */

export async function copyTextToClipboard(text: string): Promise<boolean> {
  try {
    if (navigator?.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
      return true;
    }
    // Fallback para ambientes restritos
    const textArea = document.createElement("textarea");
    textArea.value = text;
    textArea.style.position = "fixed";
    textArea.style.opacity = "0";
    document.body.appendChild(textArea);
    textArea.focus();
    textArea.select();
    const successful = document.execCommand("copy");
    document.body.removeChild(textArea);
    return successful;
  } catch (err) {
    console.warn("Falha ao copiar para a área de transferência:", err);
    return false;
  }
}

export function createClipboardFeedback(
  setFeedback: (id: string | null) => void,
  timeoutMs = 2000
) {
  let timer: ReturnType<typeof setTimeout> | null = null;

  return async (text: string, id: string): Promise<boolean> => {
    const success = await copyTextToClipboard(text);
    if (success) {
      if (timer) clearTimeout(timer);
      setFeedback(id);
      timer = setTimeout(() => {
        setFeedback(null);
        timer = null;
      }, timeoutMs);
    }
    return success;
  };
}
