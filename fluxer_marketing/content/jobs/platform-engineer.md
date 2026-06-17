<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
## Platform Engineer

Fluxer is an open-source chat app for text, voice, and communities, built for people who want a chat product that respects their time and data. The company is based in Sweden, and our employees and contractors work remotely across countries. We are a small team, so people are expected to own their work, communicate clearly, and stay close to the users affected by their decisions. The [code is public](https://github.com/fluxerapp/fluxer), so you can read how the product is built before you apply.

### What this role is

Platform engineers keep Fluxer's production systems reliable, observable, and straightforward to operate. The work covers real-time messaging, voice, media pipelines, queues, storage, deploys, certificates, databases, and the dashboards that help the team understand what is happening.

The backend services are TypeScript and Rust, the real-time infrastructure is Erlang/OTP, and performance-sensitive parts of the stack tend to be Rust. We expect platform work to include ownership. If you ship a deploy pipeline, the rollback story has to be clear. If an alert pages the team, acknowledge it and then make the system clearer or more reliable.

### What you would actually be doing

- Keeping CI/CD reliable, repeatable, and reversible
- Running container orchestration, deploys, rollbacks, migrations, and the operational details around them
- Tuning and recovering production databases, relational and wide-column, including during incidents
- Operating the real-time path, which runs on Erlang/OTP: WebSocket fan-out, voice media servers and SFUs, TURN and STUN, and edge points of presence
- Building reliability targets, capacity plans, and alerts that still mean something a month later
- Making metrics, logs, traces, and dashboards useful before an incident
- Writing runbooks that are accurate when someone needs them under pressure
- Investigating performance problems across services, queues, storage, and networks
- Owning secrets, certificates, DNS, TLS, and the security basics that keep production healthy
- Running post-incident reviews and following through on the action items

Most of this ships as visible commits in the public repo, so clear changes and clear explanations matter.

### What makes someone good at this

- You have spent real time in infrastructure, platform engineering, SRE, or production operations
- You are at home in Linux and a terminal, and comfortable with containers and deployment automation
- You understand distributed systems and the failure modes that come with them
- You have a working relationship with observability tools and the patience to make them useful
- When something breaks, you can separate evidence from guesses and move the team toward a fix

### Other things we would be glad to see

- Experience operating Erlang/OTP in production, or another actor runtime such as Elixir or Akka
- Reproducible build systems such as Nix
- Reading or writing Rust and TypeScript comfortably, since that is most of what you would be running
- S3-compatible object storage, CDN configuration, or experience delivering large amounts of media
- Real-time infrastructure depth: WebRTC, SFUs, media servers, and the specific patience they require
- The kind of network debugging where the answer turns out, once again, to be certificates
- On-call experience, especially the kind that left the system better than you found it

### Who you would work with

You would work alongside the engineers building the app, and with support, safety, and legal whenever an infrastructure decision affects users, policy, or compliance. Small team, real ownership, and a strong bias toward fixing causes rather than learning to live with symptoms.
