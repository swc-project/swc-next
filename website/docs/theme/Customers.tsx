import { ArrowUpRight } from "lucide-react";
import customers from "./customers.json";

// Names, logos, and links from https://swc.rs/docs/customers.
export function Customers() {
  return (
    <div className="team-members customer-list rp-not-doc">
      {customers.map(({ caption, image, infoLink }) => (
        <a
          className="team-member customer"
          href={infoLink}
          target="_blank"
          rel="noreferrer"
          key={infoLink}
        >
          <span className="customer__logo">
            <img
              src={`https://swc.rs${image}`}
              alt=""
              width="128"
              height="48"
              loading="lazy"
              decoding="async"
            />
          </span>
          <span className="team-member__details">
            <strong>{caption}</strong>
          </span>
          <ArrowUpRight size={15} />
        </a>
      ))}
    </div>
  );
}
