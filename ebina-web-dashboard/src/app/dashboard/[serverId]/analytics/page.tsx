'use client';

import { useEffect, useState } from 'react';
import { useParams } from 'next/navigation';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip } from 'recharts';
import { ChartContainer, type ChartConfig } from '@/components/ui/chart';
import { Button } from '@/components/ui/button';

const chartConfig = {
  count: {
    label: "Messages",
    color: "#8884d8",
  },
} satisfies ChartConfig

export default function AnalyticsPage() {
    const params = useParams();
    const serverId = params.serverId;
    const [data, setData] = useState<any[] | null>(null);
    const [loading, setLoading] = useState(true);
    const [timeframe, setTimeframe] = useState('7d');

    useEffect(() => {
        if (!serverId) return;

        async function fetchData() {
            setLoading(true);
            try {
                const res = await fetch(`/api/guilds/${serverId}/messages?timeframe=${timeframe}`);
                const jsonData = await res.json();
                setData(jsonData);
            } catch (error) {
                console.error('Failed to fetch analytics data:', error);
            } finally {
                setLoading(false);
            }
        }

        fetchData();
    }, [serverId, timeframe]);

    return (
        <div className="space-y-4">
            <h1 className="text-2xl font-bold">Server Analytics</h1>
            <div className="flex space-x-2">
                <Button onClick={() => setTimeframe('7d')}>Last 7 Days</Button>
                <Button onClick={() => setTimeframe('30d')}>Last 30 Days</Button>
            </div>
            {loading ? (
                <div className="flex items-center justify-center h-[300px]">
                    <p>Loading...</p>
                </div>
            ) : (
                <ChartContainer config={chartConfig} className="w-full h-[300px]">
                    <LineChart data={data}>
                        <CartesianGrid strokeDasharray="3 3" />
                        <XAxis dataKey="date" />
                        <YAxis />
                        <Tooltip />
                        <Line type="monotone" dataKey="count" stroke="var(--color-count)" />
                    </LineChart>
                </ChartContainer>
            )}
        </div>
    );
}
