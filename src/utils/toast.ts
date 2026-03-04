export function showToast(
  message: string,
  type: "success" | "error" | "warning" | "info" = "info",
  duration = 3000
) {
  window.dispatchEvent(
    new CustomEvent("app-toast", {
      detail: { message, type, duration },
    })
  );
}