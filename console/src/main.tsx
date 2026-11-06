import { render } from "preact";
import { ToastProvider } from "./components/shared/ui/Toast.view";
import { AppRoutes } from "./routes/AppRoutes";
import "./style.css";

const root = document.getElementById("app");

if (!root) {
  throw new Error("The application root element is missing.");
}

render(
  <ToastProvider>
    <AppRoutes />
  </ToastProvider>,
  root,
);
