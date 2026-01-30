import "./main.css";
import { RouterProvider } from "@tanstack/react-router";
import React from "react";
import ReactDOM from "react-dom/client";
import { router } from "./router";
import {TanStackDevtools} from "@tanstack/react-devtools";
import {formDevtoolsPlugin} from "@tanstack/react-form-devtools";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <RouterProvider router={router} />

      <TanStackDevtools plugins={[formDevtoolsPlugin()]} />
  </React.StrictMode>,
);
