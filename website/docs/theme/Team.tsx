import { ArrowUpRight } from "lucide-react";

// Names, roles, and profiles from https://swc.rs/docs/team.
const members = [
  { name: "kdy1 / Donny", username: "kdy1", role: "Lead Maintainer" },
  { name: "OJ Kwon", username: "kwonoj", role: "Core Team" },
  { name: "magic-akari", username: "magic-akari", role: "Core Team" },
  { name: "Austaras", username: "Austaras", role: "Core Team" },
  { name: "CPunisher", username: "CPunisher", role: "Core Team" },
  { name: "bohan", username: "bvanjoi", role: "Core Team" },
  { name: "LongYinan", username: "Brooooooklyn", role: "Maintainer of @swc-node/*" },
];

export function TeamMembers() {
  return (
    <div className="team-members rp-not-doc">
      {members.map(({ name, username, role }) => (
        <a
          className="team-member"
          href={`https://github.com/${username}`}
          target="_blank"
          rel="noreferrer"
          key={username}
        >
          <img
            src={`https://github.com/${username}.png?size=128`}
            alt={`${name} avatar`}
            width="48"
            height="48"
            loading="lazy"
            decoding="async"
          />
          <span className="team-member__details">
            <strong>{name}</strong>
            <span>{role}</span>
          </span>
          <ArrowUpRight size={15} />
        </a>
      ))}
    </div>
  );
}
