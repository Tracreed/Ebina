import { getServerSession } from "next-auth/next"
import { authOptions } from "@/app/api/auth/[...nextauth]/route"
import { redirect } from "next/navigation"

export default async function Dashboard() {
  const session = await getServerSession(authOptions)

  if (!session) {
    redirect("/")
  }

  // TODO: Fetch user's guilds from Discord API
  // TODO: Fetch bot's guilds from our API
  // TODO: Find mutual guilds

  return (
    <div>
      <h1>Select a Server</h1>
      {/* TODO: Display list of mutual guilds */}
    </div>
  )
}
