import { getServerSession } from "next-auth/next"
import { authOptions } from "@/app/api/auth/[...nextauth]/route"
import { redirect } from "next/navigation"

export default async function ServerDashboard({ params }: { params: { serverId: string } }) {
  const session = await getServerSession(authOptions)

  if (!session) {
    redirect("/")
  }

  // TODO: Fetch server stats from our API using params.serverId

  return (
    <div>
      <h1>Server Dashboard</h1>
      <p>Server ID: {params.serverId}</p>
      {/* TODO: Display server stats */}
    </div>
  )
}
