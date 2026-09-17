import { Navigate, Route, Routes } from "react-router-dom";
import DisplayPage from "./pages/DisplayPage";
import OperatorPage from "./pages/OperatorPage";

export default function App() {
  return (
    <Routes>
      <Route path="/display/:code" element={<DisplayPage />} />
      <Route path="/" element={<OperatorPage />} />
      <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
  );
}
