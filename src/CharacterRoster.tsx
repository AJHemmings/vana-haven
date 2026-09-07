export type Character = {
  game_character_id: number;
  name: string;
  last_seen_at: string;
};

type Props = { characters: Character[] };

export default function CharacterRoster({ characters }: Props) {
  if (characters.length === 0) {
    return (
      <p className="text-neutral-400">
        No characters yet — load the Vana Haven addon in-game to get started.
      </p>
    );
  }

  return (
    <ul className="divide-y divide-neutral-800">
      {characters.map((character) => (
        <li key={character.game_character_id} className="py-2">
          <span className="font-medium">{character.name}</span>
        </li>
      ))}
    </ul>
  );
}
