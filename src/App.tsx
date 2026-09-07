import { useEffect, useState } from "react";
import CharacterRoster, { type Character } from "./CharacterRoster";
import { fetchCharacters, onCharacterUpdated } from "./bridge";

export default function App() {
  const [characters, setCharacters] = useState<Character[]>([]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    fetchCharacters().then(setCharacters);
    onCharacterUpdated(() => {
      fetchCharacters().then(setCharacters);
    }).then((fn) => {
      unlisten = fn;
    });

    return () => unlisten?.();
  }, []);

  return (
    <main className="min-h-screen bg-neutral-950 text-neutral-100 p-6">
      <h1 className="text-xl font-semibold mb-4">Vana Haven</h1>
      <CharacterRoster characters={characters} />
    </main>
  );
}
