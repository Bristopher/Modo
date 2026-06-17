> The Fluxer Bug Hunter profile badge is currently only available as a reward for responsible disclosure through our [security bug bounty programme](/security), not for general bug reports submitted via this guide.

Use this guide to put together a clear report so we can reproduce and fix the bug quickly. Screenshots, short screen recordings, and relevant logs or files help us diagnose the issue faster.

## Bug report template

Give your report a specific title, for example "Media upload stalls at 95%". Then include the following sections:

- Steps to reproduce: number each step, and include the exact clicks or taps, inputs, shortcuts, and any timing or ordering details.
- Expected result: what you expected to happen.
- Actual result: what happened instead, including the exact error or on-screen message.
- System and client settings: in User Settings, tap your client info at the bottom of the sidebar to copy it, then paste it into the report.

## Add evidence

Include anything that shows the issue: screenshots, short videos, logs, or sample files and exports. The more specific your report is, the faster we can help.

### A note on privacy when sharing evidence

Only include data that is needed to demonstrate the bug. Screenshots and screen recordings can capture more than you intend: other people's usernames and avatars, message content from third parties in DMs or Communities, open tabs in the background, notification previews, tokens visible in network logs.

Before sending:

- Crop or blur unrelated chats, friend lists, and Community names.
- If you share a log file, search it for email addresses, IP addresses, session tokens, and other people's user IDs, and redact anything you do not specifically need to include.
- Prefer screenshots over photos. Photos often carry hidden EXIF metadata such as GPS coordinates.
- Never paste your own session token, password, or authentication cookie. We never need them to investigate a bug.

## Submit your report

Email [bugs@fluxer.app](mailto:bugs@fluxer.app) with the completed template. A concise but descriptive subject line helps us triage quickly. If you prefer GitHub, you are welcome to file issues in the [Fluxer GitHub repository](https://github.com/fluxerapp/fluxer).

## Security issues

If you believe the issue is security-related, visit our [security bug bounty page](/security) instead of emailing support. Follow the guidance there, and include clear steps, why you believe it is a security risk, and any impact you have identified. We respond quickly to assess the report, coordinate a fix, and discuss disclosure expectations.
