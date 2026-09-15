export default function ItemPlaceholderIcon({ name }: { name: string }) {
  const letter = (name.match(/[A-Za-z0-9]/)?.[0] ?? "?").toUpperCase();
  return (
    <div
      className="w-10 h-10 flex items-center justify-center rounded bg-neutral-800 text-neutral-200 font-semibold"
      aria-hidden="true"
    >
      {letter}
    </div>
  );
}
