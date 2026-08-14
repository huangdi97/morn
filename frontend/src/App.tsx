import { Navigate, Route, Routes } from "react-router-dom";
import { Shell } from "./components/Shell";
import ConsolePage from "./pages/ConsolePage";
import Hub from "./pages/Hub";
import Studio from "./pages/Studio";
import Workbench from "./pages/Workbench";

export default function App() {
  return (
    <Shell>
      <Routes>
        <Route path="/" element={<Workbench />} />
        <Route path="/workbench" element={<Workbench />} />
        <Route path="/studio" element={<Studio />} />
        <Route path="/console" element={<ConsolePage />} />
        <Route path="/hub" element={<Hub />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </Shell>
  );
}
