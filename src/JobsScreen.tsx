import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import {
  fetchCharacter,
  fetchCharacterJobs,
  onCharacterUpdated,
  type CharacterDetail,
  type JobLevel,
} from "./bridge";
import { JOBS, jobAbbreviation } from "./jobs";

export default function JobsScreen() {
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
        .catch((err) => console.error("[vana-haven] failed to fetch character jobs", err));
    };

    load();
    onCharacterUpdated(load).then((fn) => {
      unlisten = fn;
    });

    return () => unlisten?.();
  }, [id]);

  const levelFor = (jobId: number) => jobs.find((j) => j.job_id === jobId)?.level ?? 0;

  const ready = character != null && character.main_job_id != null && jobs.length > 0;

  const mainSubLine =
    ready && character.sub_job_id
      ? `${jobAbbreviation(character.main_job_id!)} Lv.${levelFor(character.main_job_id!)} / ${jobAbbreviation(character.sub_job_id)} Lv.${levelFor(character.sub_job_id)}`
      : ready
        ? `${jobAbbreviation(character.main_job_id!)} Lv.${levelFor(character.main_job_id!)}`
        : null;

  return (
    <main className="min-h-screen bg-neutral-950 text-neutral-100 p-6">
      <Link to={`/character/${id}`} className="text-neutral-400 hover:underline">
        ← back
      </Link>
      <h1 className="text-xl font-semibold mt-2">Jobs</h1>
      {!ready ? (
        <p className="text-neutral-400 mb-4">waiting for job data...</p>
      ) : (
        <>
          <p className="text-neutral-400 mb-4">{mainSubLine}</p>
          <ul className="divide-y divide-neutral-800">
            {JOBS.map((job) => {
              const entry = jobs.find((j) => j.job_id === job.id);
              return (
                <li key={job.id} className="py-2 flex justify-between">
                  <span>{job.abbreviation}</span>
                  <span>
                    Lv. {entry?.level ?? 0} &middot; ML {entry?.master_level ?? 0}
                  </span>
                </li>
              );
            })}
          </ul>
        </>
      )}
    </main>
  );
}
