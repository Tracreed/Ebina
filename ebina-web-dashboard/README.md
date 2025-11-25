# Ebina Web Dashboard

This is the web dashboard for the Ebina Discord bot, built with Next.js, Tailwind CSS, and shadcn/ui.

## Getting Started

First, you'll need to set up your environment variables. Copy the `.env.example` file to a new file named `.env.local`:

```bash
cp .env.example .env.local
```

Next, open `.env.local` and fill in the required values for `DISCORD_CLIENT_ID`, `DISCORD_CLIENT_SECRET`, and `NEXTAUTH_SECRET`.

Then, install the dependencies:

```bash
npm install
```

Finally, run the development server:

```bash
npm run dev
```

Open [http://localhost:3000](http://localhost:3000) with your browser to see the result.
