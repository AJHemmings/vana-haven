import { HashRouter, Routes, Route } from "react-router-dom";
import Roster from "./Roster";
import CharacterHub from "./CharacterHub";
import JobsScreen from "./JobsScreen";

export default function App() {
  return (
    <HashRouter>
      <Routes>
        <Route path="/" element={<Roster />} />
        <Route path="/character/:gameCharacterId" element={<CharacterHub />} />
        <Route path="/character/:gameCharacterId/jobs" element={<JobsScreen />} />
      </Routes>
    </HashRouter>
  );
}
