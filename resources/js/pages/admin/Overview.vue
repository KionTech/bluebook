<script setup lang="ts">
import { index as overviewRoute } from '@/actions/App/Http/Controllers/Admin/AdministrationController';
import { index as pilotsRoute } from '@/actions/App/Http/Controllers/Admin/PilotController';
import { edit as settingsRoute } from '@/actions/App/Http/Controllers/Admin/SettingsController';
import { index as requestsRoute } from '@/actions/App/Http/Controllers/Admin/StandingRequestController';
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar';
import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
} from '@/components/ui/card';
import { useInitials } from '@/composables/useInitials';
import { countdown, dateWithAgo } from '@/lib/date';
import { eveImage, standingLabel, standingTextClass } from '@/lib/eve';
import AdminLayout from '@/layouts/admin/Layout.vue';
import AppLayout from '@/layouts/AppLayout.vue';
import type { BreadcrumbItem } from '@/types';
import { Head, Link, usePoll } from '@inertiajs/vue3';
import { useIntervalFn, useNow } from '@vueuse/core';
import { ArrowRight } from '@lucide/vue';
import { computed } from 'vue';

const REFRESH_INTERVAL_MS = 5 * 60 * 1000;

type EffectiveStanding = {
    standing: number;
    source: 'direct' | 'corporation' | 'alliance';
    via_type: string;
    via_id: number;
    via_name: string | null;
};

type StandingRequestItem = {
    id: number;
    status: string;
    created_at: string;
    subject: { type: string; id: number; name: string | null };
    requested_by: string;
    effective_standing: EffectiveStanding | null;
};

const props = defineProps<{
    source: {
        type: string;
        entity_id: number;
        entity_name: string | null;
        last_synced_at: string | null;
    } | null;
    stats: {
        pending_requests: number;
        pilots: number;
        syncing_characters: number;
        source_contacts: number;
    };
    recentRequests: StandingRequestItem[];
}>();

const breadcrumbs: BreadcrumbItem[] = [
    { title: 'Administration', href: overviewRoute() },
];

// Refresh server props every 30s so incoming standing requests appear live.
usePoll(30000);

const { getInitials } = useInitials();

// A clock that ticks every second so the relative times stay live.
const now = useNow({ scheduler: (callback) => useIntervalFn(callback, 1000) });

const lastRefreshed = computed(() =>
    dateWithAgo(props.source?.last_synced_at, now.value.getTime()),
);

const nextRefresh = computed(() => {
    if (!props.source?.last_synced_at) {
        return null;
    }

    const target =
        new Date(props.source.last_synced_at).getTime() + REFRESH_INTERVAL_MS;

    return countdown(target, now.value.getTime());
});

const tiles = computed(() => [
    {
        label: 'Pending requests',
        value: props.stats.pending_requests,
        hint:
            props.stats.pending_requests === 0
                ? 'All caught up'
                : 'Waiting for a decision',
        href: requestsRoute(),
        attention: props.stats.pending_requests > 0,
    },
    {
        label: 'Pilots',
        value: props.stats.pilots,
        hint: 'Registered accounts',
        href: pilotsRoute(),
        attention: false,
    },
    {
        label: 'Syncing characters',
        value: props.stats.syncing_characters,
        hint: 'Mirroring the source',
        href: pilotsRoute(),
        attention: false,
    },
    {
        label: 'Source contacts',
        value: props.stats.source_contacts,
        hint: 'Canonical standings',
        href: settingsRoute(),
        attention: false,
    },
]);

function subjectImage(type: string, id: number): string {
    return eveImage(type, id) ?? '';
}
</script>

<template>
    <Head title="Administration" />

    <AppLayout :breadcrumbs="breadcrumbs">
        <AdminLayout
            title="Overview"
            description="The state of the standings service at a glance."
        >
            <!-- Highlights -->
            <div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
                <Link
                    v-for="tile in tiles"
                    :key="tile.label"
                    :href="tile.href"
                    class="group relative overflow-hidden rounded-xl border bg-card p-5 transition-colors hover:bg-muted/50"
                >
                    <span
                        v-if="tile.attention"
                        class="absolute inset-x-0 top-0 h-0.5 bg-amber-500"
                    />
                    <p class="text-sm text-muted-foreground">
                        {{ tile.label }}
                    </p>
                    <p
                        class="mt-2 text-3xl font-semibold tracking-tight tabular-nums"
                    >
                        {{ tile.value }}
                    </p>
                    <p
                        class="mt-1 flex items-center gap-1 text-xs text-muted-foreground"
                    >
                        {{ tile.hint }}
                        <ArrowRight
                            class="h-3 w-3 opacity-0 transition-opacity group-hover:opacity-100"
                        />
                    </p>
                </Link>
            </div>

            <div class="grid gap-6 lg:grid-cols-2">
                <!-- Source status -->
                <Card>
                    <CardHeader>
                        <CardTitle>Standings source</CardTitle>
                        <CardDescription>
                            The entity every character mirrors.
                        </CardDescription>
                    </CardHeader>
                    <CardContent>
                        <div v-if="source" class="flex items-center gap-4">
                            <Avatar
                                class="h-12 w-12 shrink-0 overflow-hidden rounded-lg"
                            >
                                <AvatarImage
                                    :src="
                                        subjectImage(
                                            source.type,
                                            source.entity_id,
                                        )
                                    "
                                    :alt="source.entity_name ?? ''"
                                />
                                <AvatarFallback
                                    class="rounded-lg text-xs uppercase"
                                >
                                    {{ source.type.slice(0, 2) }}
                                </AvatarFallback>
                            </Avatar>
                            <div class="min-w-0 flex-1">
                                <p class="truncate text-sm font-medium">
                                    {{ source.entity_name ?? source.entity_id }}
                                </p>
                                <p
                                    class="text-xs text-muted-foreground capitalize"
                                >
                                    {{ source.type }}
                                </p>
                            </div>
                            <div class="text-right text-xs">
                                <p class="text-muted-foreground">
                                    Refreshed {{ lastRefreshed }}
                                </p>
                                <p
                                    v-if="nextRefresh"
                                    class="mt-0.5 font-medium tabular-nums"
                                >
                                    Next in {{ nextRefresh }}
                                </p>
                            </div>
                        </div>
                        <p v-else class="text-sm text-muted-foreground">
                            No source is configured yet. Set one in the settings
                            to start syncing.
                        </p>
                        <Link
                            :href="settingsRoute()"
                            class="mt-4 inline-flex items-center gap-1 text-sm text-muted-foreground transition-colors hover:text-foreground"
                        >
                            Manage source
                            <ArrowRight class="h-3.5 w-3.5" />
                        </Link>
                    </CardContent>
                </Card>

                <!-- Recent pending requests -->
                <Card>
                    <CardHeader>
                        <CardTitle>Pending requests</CardTitle>
                        <CardDescription>
                            The latest standing requests waiting for a decision.
                        </CardDescription>
                    </CardHeader>
                    <CardContent>
                        <p
                            v-if="recentRequests.length === 0"
                            class="py-6 text-center text-sm text-muted-foreground"
                        >
                            Nothing waiting. New requests appear here.
                        </p>
                        <ul v-else class="-mx-2">
                            <li
                                v-for="request in recentRequests"
                                :key="request.id"
                                class="flex items-center gap-3 rounded-md px-2 py-2 transition-colors odd:bg-muted/40"
                            >
                                <Avatar
                                    class="h-8 w-8 shrink-0 overflow-hidden rounded"
                                >
                                    <AvatarImage
                                        :src="
                                            subjectImage(
                                                request.subject.type,
                                                request.subject.id,
                                            )
                                        "
                                        :alt="request.subject.name ?? ''"
                                    />
                                    <AvatarFallback class="rounded text-xs">
                                        {{
                                            getInitials(
                                                request.subject.name ?? '?',
                                            )
                                        }}
                                    </AvatarFallback>
                                </Avatar>
                                <div class="min-w-0 flex-1">
                                    <p class="truncate text-sm font-medium">
                                        {{
                                            request.subject.name ??
                                            request.subject.id
                                        }}
                                    </p>
                                    <p
                                        class="truncate text-xs text-muted-foreground"
                                    >
                                        Requested by {{ request.requested_by }}
                                    </p>
                                </div>
                                <span
                                    v-if="request.effective_standing"
                                    class="shrink-0 text-xs font-medium"
                                    :class="
                                        standingTextClass(
                                            request.effective_standing.standing,
                                        )
                                    "
                                >
                                    {{
                                        standingLabel(
                                            request.effective_standing.standing,
                                        )
                                    }}
                                </span>
                            </li>
                        </ul>
                        <Link
                            :href="requestsRoute()"
                            class="mt-4 inline-flex items-center gap-1 text-sm text-muted-foreground transition-colors hover:text-foreground"
                        >
                            Review requests
                            <ArrowRight class="h-3.5 w-3.5" />
                        </Link>
                    </CardContent>
                </Card>
            </div>
        </AdminLayout>
    </AppLayout>
</template>
