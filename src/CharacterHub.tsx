import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import {
  fetchCharacter,
  fetchCharacterJobs,
  onCharacterUpdated,
  type CharacterDetail,
  type JobLevel,
} from "./bridge";
import { jobAbbreviation } from "./jobs";

export default function CharacterHub() {
  const { gameCharacterId } = useParams<{ gameCharacterId: string }>();
  const id = Number(gameCharacterId);
  const [character, setCharacter] = useState<CharacterDetail | null>(null);
  const [jobs, setJobs] = useState<JobLevel[]>([]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const load = () => {
      Promise.all([fetchCharacter(id), fetchCharacterJobs(id)])
        .then(([characterResult, jobsResult]) => {
          setCharacter(characterResult);
          setJobs(jobsResult);
        })
        .catch((err) => console.error("[vana-haven] failed to fetch character", err));
    };

    load();
    onCharacterUpdated(load).then((fn) => {
      unlisten = fn;
    });

    return () => unlisten?.();
  }, [id]);

  if (!character) {
    return <p className="text-neutral-400">waiting for job data...</p>;
  }

  // Level isn't stored on CharacterDetail (per spec §4) — looked up from the
  // jobs array by id, same as the Jobs screen does for every other job.
  const jobWithLevel = (jobId: number) => {
    const level = jobs.find((j) => j.job_id === jobId)?.level ?? 0;
    return `${jobAbbreviation(jobId)} Lv.${level}`;
  };

  const mainSummary =
    character.main_job_id == null
      ? "waiting for job data..."
      : character.sub_job_id
        ? `${jobWithLevel(character.main_job_id)} / ${jobWithLevel(character.sub_job_id)}`
        : jobWithLevel(character.main_job_id);

  return (
    <main className="min-h-screen bg-neutral-950 text-neutral-100 p-6">
      <Link to="/" className="text-neutral-400 hover:underline">
        ← back to roster
      </Link>
      <h1 className="text-xl font-semibold mt-2">{character.name}</h1>
      <p className="text-neutral-400 mb-4">{mainSummary}</p>
      <Link
        to={`/character/${id}/jobs`}
        className="inline-block px-3 py-1.5 rounded bg-neutral-800 hover:bg-neutral-700"
      >
        Jobs
      </Link>
    </main>
  );
}
