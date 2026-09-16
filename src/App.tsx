import { HashRouter, Routes, Route } from "react-router-dom";
import Roster from "./Roster";
import CharacterHub from "./CharacterHub";
import JobsScreen from "./JobsScreen";
import GearScreen from "./GearScreen";
import KeyItemsScreen from "./KeyItemsScreen";

export default function App() {
  return (
    <HashRouter>
      <Routes>
        <Route path="/" element={<Roster />} />
        <Route path="/character/:gameCharacterId" element={<CharacterHub />} />
        <Route path="/character/:gameCharacterId/jobs" element={<JobsScreen />} />
        <Route path="/character/:gameCharacterId/jobs/:jobId/gear" element={<GearScreen />} />
        <Route path="/character/:gameCharacterId/key-items" element={<KeyItemsScreen />} />
      </Routes>
    </HashRouter>
  );
}
