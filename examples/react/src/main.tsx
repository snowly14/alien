import { createRoot } from "react-dom/client";

import { Button } from "./button";
import "./base.css";

const root = document.getElementById("root");
if (!root) throw new Error("Missing React root");

createRoot(root).render(
  <main>
    <h1>Alien avec React</h1>
    <p>Survole le bouton et utilise Tab pour découvrir ses états compilés par Alien.</p>
    <div className="buttons">
      <Button />
    </div>
  </main>,
);
