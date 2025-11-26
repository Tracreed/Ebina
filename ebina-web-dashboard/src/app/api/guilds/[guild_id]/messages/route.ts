import { NextRequest, NextResponse } from 'next/server';
import { sql } from 'sqlx-ts';

export async function GET(
    request: NextRequest,
    { params }: { params: Promise<{ guild_id: string }> }
) {
    const { guild_id: guildId } = await params;
    const { searchParams } = new URL(request.url);
    const timeframe = searchParams.get('timeframe') || '7d';

    let startDate: Date;
    const now = new Date();

    switch (timeframe) {
        case '30d':
            startDate = new Date(now.setDate(now.getDate() - 30));
            break;
        case '7d':
        default:
            startDate = new Date(now.setDate(now.getDate() - 7));
            break;
    }

    try {
        // This assumes that you have set up the sqlx-ts helper and it can connect to your database.
        // You will need to configure the database connection string in your environment variables.
        const messageCounts = await sql`
            SELECT date, count
            FROM message_counts
            WHERE server_id = ${guildId} AND date >= ${startDate.toISOString().split('T')[0]}
            ORDER BY date ASC
        `;

        return NextResponse.json(messageCounts);
    } catch (error) {
        console.error('Failed to fetch message counts:', error);
        return new NextResponse('Internal Server Error', { status: 500 });
    }
}
