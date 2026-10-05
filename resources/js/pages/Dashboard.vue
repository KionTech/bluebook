<script setup lang="ts">
import {
    sync as syncCharacters,
    update as updateCharacterSync,
} from '@/actions/App/Http/Controllers/CharacterSyncController';
import { update as setMainCharacterAction } from '@/actions/App/Http/Controllers/MainCharacterController';
import { store as requestStandingAction } from '@/actions/App/Http/Controllers/StandingRequestController';
import SetupWizard from '@/components/SetupWizard.vue';
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
} from '@/components/ui/card';
import {
    Dialog,
    DialogClose,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Switch } from '@/components/ui/switch';
import { useInitials } from '@/composables/useInitials';
import { countdown, dateWithAgo } from '@/lib/date';
import { eveImage, standingLabel, standingTextClass } from '@/lib/eve';
import AppLayout from '@/layouts/AppLayout.vue';
import { dashboard } from '@/routes';
import type { BreadcrumbItem } from '@/types';
import { Head, router, usePage, usePoll } from '@inertiajs/vue3';
import { useIntervalFn, useNow } from '@vueuse/core';
import { KeyRound, Lock, Plus, RefreshCw, Search, Star } from '@lucide/vue';
import { computed, ref } from 'vue';

const REFRESH_INTERVAL_MS = 5 * 60 * 1000;

// Refresh server props every 30s so standings and characters stay live.
usePoll(30000);

type Standing = {
    contact_id: number;
    contact_type: string;
    name: string | null;
    standing: number;
    redundant_via: {
        contact_type: string;
        contact_id: number;
        name: string | null;
    } | null;
};

type EntitySummary = {
    id: number;
    name: string | null;
    ticker: string | null;
};

type RequestOption = {
    type: string;
    id: number;
    name: string | null;
    via_character_id: number;
    status: string | null;
};

type SyncCharacter = {
    id: number;
    name: string;
    is_main: boolean;
    should_sync: boolean;
    has_write_scope: boolean;
    inherits_source: boolean;
    synced_contacts_count: number;
    corporation: EntitySummary | null;
    alliance: EntitySummary | null;
};

const props = defineProps<{
    showSetupWizard: boolean;
    source: {
        type: string;
        entity_id: number;
        entity_name: string | null;
        last_synced_at: string | null;
    } | null;
    canViewStandings: boolean;
    standings: Standing[] | null;
    characters: SyncCharacter[];
    requestableOptions: RequestOption[];
}>();

const breadcrumbs: BreadcrumbItem[] = [
    { title: 'Dashboard', href: dashboard() },
];

const { getInitials } = useInitials();

const page = usePage();
const syncScopesUrl = computed(() => page.props.auth.sync_scopes_url);

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

// Within a standing level, order entity types consistently.
const TYPE_ORDER = ['corporation', 'alliance', 'character', 'faction'];

function typeRank(type: string): number {
    const index = TYPE_ORDER.indexOf(type);
    return index === -1 ? TYPE_ORDER.length : index;
}

const sortedStandings = computed(() =>
    [...(props.standings ?? [])].sort(
        (a, b) =>
            b.standing - a.standing ||
            typeRank(a.contact_type) - typeRank(b.contact_type) ||
            (a.name ?? `${a.contact_id}`).localeCompare(
                b.name ?? `${b.contact_id}`,
                undefined,
                { sensitivity: 'base' },
            ),
    ),
);

const standingsSearch = ref('');

const filteredStandings = computed(() => {
    const query = standingsSearch.value.trim().toLowerCase();

    if (query === '') {
        return sortedStandings.value;
    }

    return sortedStandings.value.filter((standing) =>
        (standing.name ?? `${standing.contact_id}`)
            .toLowerCase()
            .includes(query),
    );
});

function syncMyCharacters(): void {
    router.post(syncCharacters.url(), {}, { preserveScroll: true });
}

function setMainCharacter(character: SyncCharacter): void {
    router.put(
        setMainCharacterAction.url({ character: character.id }),
        {},
        { preserveScroll: true },
    );
}

function toggleSync(character: SyncCharacter, value: boolean): void {
    router.put(
        updateCharacterSync.url({ character: character.id }),
        { should_sync: value },
        { preserveScroll: true },
    );
}

function requestStanding(option: RequestOption): void {
    if (!canRequest(option)) {
        return;
    }

    router.post(
        requestStandingAction.url({ character: option.via_character_id }),
        { type: option.type },
        { preserveScroll: true },
    );
}

// Whether an entity is already in the current standings list shown on this page.
function hasStanding(option: RequestOption): boolean {
    return (props.standings ?? []).some(
        (standing) =>
            standing.contact_type === option.type &&
            standing.contact_id === option.id,
    );
}

function canRequest(option: RequestOption): boolean {
    return (
        !hasStanding(option) &&
        option.status !== 'pending' &&
        option.status !== 'done'
    );
}

function optionStatusLabel(option: RequestOption): string | null {
    if (hasStanding(option)) {
        return 'On the list';
    }
    if (option.status === 'pending') {
        return 'Requested';
    }
    if (option.status === 'done') {
        return 'Approved';
    }
    if (option.status === 'rejected') {
        return 'Rejected';
    }
    return null;
}
</script>

<template>
    <Head title="Standings" />

    <AppLayout :breadcrumbs="breadcrumbs">
        <SetupWizard v-if="showSetupWizard" :characters="characters" />
        <div class="flex flex-1 flex-col gap-6 p-4">
            <!-- Source -->
            <Card>
                <CardHeader class="flex flex-row items-center gap-3">
                    <Avatar
                        v-if="source"
                        class="h-10 w-10 shrink-0 overflow-hidden rounded"
                    >
                        <AvatarImage
                            v-if="eveImage(source.type, source.entity_id)"
                            :src="eveImage(source.type, source.entity_id)!"
                            :alt="source.entity_name ?? ''"
                        />
                        <AvatarFallback class="rounded text-xs uppercase">
                            {{ source.type.slice(0, 2) }}
                        </AvatarFallback>
                    </Avatar>
                    <div>
                        <CardTitle>Standings source</CardTitle>
                        <CardDescription>
                            <template v-if="source">
                                Mirroring
                                <span class="font-medium text-foreground">{{
                                    source.entity_name ?? source.type
                                }}</span
                                >'s {{ source.type }} contacts onto every
                                opted-in character.
                            </template>
                            <template v-else>
                                No source has been configured yet.
                            </template>
                        </CardDescription>
                    </div>
                </CardHeader>
                <CardContent>
                    <dl class="grid gap-3 text-sm sm:grid-cols-3">
                        <div v-if="standings">
                            <dt class="text-muted-foreground">Standings</dt>
                            <dd class="font-medium">{{ standings.length }}</dd>
                        </div>
                        <div>
                            <dt class="text-muted-foreground">
                                Last refreshed
                            </dt>
                            <dd class="font-medium">{{ lastRefreshed }}</dd>
                        </div>
                        <div v-if="nextRefresh">
                            <dt class="text-muted-foreground">Next refresh</dt>
                            <dd class="font-medium tabular-nums">
                                {{ nextRefresh }}
                            </dd>
                        </div>
                    </dl>
                </CardContent>
            </Card>

            <!-- Current standings overview -->
            <Card>
                <CardHeader>
                    <CardTitle>Current standings</CardTitle>
                    <CardDescription>
                        The canonical standings pulled from the source. These
                        are applied to every syncing character.
                    </CardDescription>
                </CardHeader>
                <CardContent>
                    <div
                        v-if="!canViewStandings"
                        class="flex flex-col items-center gap-2 py-10 text-center"
                    >
                        <Lock class="h-5 w-5 text-muted-foreground" />
                        <p class="text-sm font-medium">Standings are hidden</p>
                        <p class="max-w-sm text-sm text-muted-foreground">
                            None of your characters is eligible yet. Request a
                            standing for a character, corporation or alliance
                            below — once an admin approves it, the standings
                            unlock.
                        </p>
                    </div>
                    <p
                        v-else-if="!standings || standings.length === 0"
                        class="py-8 text-center text-sm text-muted-foreground"
                    >
                        No standings have been pulled from the source yet.
                    </p>
                    <template v-else>
                        <div class="relative mb-3">
                            <Search
                                class="absolute top-1/2 left-2.5 h-4 w-4 -translate-y-1/2 text-muted-foreground"
                            />
                            <Input
                                v-model="standingsSearch"
                                type="search"
                                name="standings-search"
                                placeholder="Search standings..."
                                aria-label="Search standings"
                                class="pl-8"
                            />
                        </div>
                        <p
                            v-if="filteredStandings.length === 0"
                            class="py-8 text-center text-sm text-muted-foreground"
                        >
                            No standings match your search.
                        </p>
                        <ul v-else class="-mx-2">
                            <li
                                v-for="standing in filteredStandings"
                                :key="standing.contact_id"
                                class="flex items-center gap-3 rounded-md px-2 py-1.5 transition-colors odd:bg-muted/40 hover:bg-muted"
                            >
                                <Avatar
                                    class="h-5 w-5 shrink-0 overflow-hidden rounded-sm"
                                >
                                    <AvatarImage
                                        v-if="
                                            eveImage(
                                                standing.contact_type,
                                                standing.contact_id,
                                            )
                                        "
                                        :src="
                                            eveImage(
                                                standing.contact_type,
                                                standing.contact_id,
                                            )!
                                        "
                                        :alt="`${standing.contact_id}`"
                                    />
                                    <AvatarFallback
                                        class="rounded-sm text-[8px] uppercase"
                                    >
                                        {{ standing.contact_type.slice(0, 2) }}
                                    </AvatarFallback>
                                </Avatar>
                                <div class="min-w-0 flex-1">
                                    <p class="truncate text-sm font-medium">
                                        {{
                                            standing.name ?? standing.contact_id
                                        }}
                                    </p>
                                    <p
                                        v-if="standing.redundant_via"
                                        class="truncate text-xs text-amber-600 dark:text-amber-400"
                                        :title="`This standing already applies through the ${standing.redundant_via.contact_type} and can be removed from the source.`"
                                    >
                                        Redundant via
                                        {{
                                            standing.redundant_via.name ??
                                            standing.redundant_via.contact_id
                                        }}
                                    </p>
                                </div>
                                <span
                                    class="flex-1 truncate text-xs text-muted-foreground capitalize"
                                >
                                    {{ standing.contact_type }}
                                </span>
                                <span
                                    class="w-12 shrink-0 text-right text-sm font-semibold tabular-nums"
                                    :class="
                                        standingTextClass(standing.standing)
                                    "
                                >
                                    {{ standingLabel(standing.standing) }}
                                </span>
                            </li>
                        </ul>
                    </template>
                </CardContent>
            </Card>

            <!-- Your characters -->
            <Card>
                <CardHeader
                    class="flex flex-row items-start justify-between gap-4"
                >
                    <div>
                        <CardTitle>Your characters</CardTitle>
                        <CardDescription>
                            Choose which characters mirror the source's
                            standings, and star your main character.
                        </CardDescription>
                    </div>
                    <div class="flex items-center gap-2">
                        <Dialog v-if="requestableOptions.length > 0">
                            <DialogTrigger as-child>
                                <Button
                                    type="button"
                                    variant="outline"
                                    size="sm"
                                >
                                    <Plus class="h-4 w-4" />
                                    Request standing
                                </Button>
                            </DialogTrigger>
                            <DialogContent class="sm:max-w-md">
                                <DialogHeader>
                                    <DialogTitle>Request standing</DialogTitle>
                                    <DialogDescription>
                                        Ask the admins to add one of your
                                        characters, corporations or alliances to
                                        the standings.
                                    </DialogDescription>
                                </DialogHeader>
                                <ul class="-mx-2 max-h-80 overflow-y-auto">
                                    <li
                                        v-for="option in requestableOptions"
                                        :key="`${option.type}:${option.id}`"
                                    >
                                        <DialogClose as-child>
                                            <button
                                                type="button"
                                                :disabled="!canRequest(option)"
                                                class="flex w-full items-center gap-3 rounded-md px-2 py-2 text-left transition-colors hover:bg-muted disabled:pointer-events-none disabled:opacity-60"
                                                @click="requestStanding(option)"
                                            >
                                                <Avatar
                                                    class="h-8 w-8 shrink-0 overflow-hidden rounded"
                                                >
                                                    <AvatarImage
                                                        v-if="
                                                            eveImage(
                                                                option.type,
                                                                option.id,
                                                            )
                                                        "
                                                        :src="
                                                            eveImage(
                                                                option.type,
                                                                option.id,
                                                            )!
                                                        "
                                                        :alt="option.name ?? ''"
                                                    />
                                                    <AvatarFallback
                                                        class="rounded text-[10px] uppercase"
                                                    >
                                                        {{
                                                            option.type.slice(
                                                                0,
                                                                2,
                                                            )
                                                        }}
                                                    </AvatarFallback>
                                                </Avatar>
                                                <div class="flex-1">
                                                    <p
                                                        class="text-sm font-medium"
                                                    >
                                                        {{
                                                            option.name ??
                                                            option.id
                                                        }}
                                                    </p>
                                                    <p
                                                        class="text-xs text-muted-foreground capitalize"
                                                    >
                                                        {{ option.type }}
                                                    </p>
                                                </div>
                                                <span
                                                    v-if="
                                                        optionStatusLabel(
                                                            option,
                                                        )
                                                    "
                                                    class="text-xs text-muted-foreground"
                                                >
                                                    {{
                                                        optionStatusLabel(
                                                            option,
                                                        )
                                                    }}
                                                </span>
                                            </button>
                                        </DialogClose>
                                    </li>
                                </ul>
                            </DialogContent>
                        </Dialog>
                        <Button
                            type="button"
                            variant="secondary"
                            size="sm"
                            @click="syncMyCharacters"
                        >
                            <RefreshCw class="h-4 w-4" />
                            Sync now
                        </Button>
                    </div>
                </CardHeader>
                <CardContent>
                    <ul class="-mx-2">
                        <li
                            v-for="character in characters"
                            :key="character.id"
                            class="flex items-center gap-4 rounded-md px-2 py-3 transition-colors odd:bg-muted/40 hover:bg-muted"
                        >
                            <Avatar class="h-9 w-9 overflow-hidden rounded">
                                <AvatarImage
                                    :src="eveImage('character', character.id)!"
                                    :alt="character.name"
                                />
                                <AvatarFallback class="rounded text-xs">
                                    {{ getInitials(character.name) }}
                                </AvatarFallback>
                            </Avatar>
                            <div class="flex-1">
                                <p
                                    class="flex items-center gap-2 text-sm font-medium"
                                >
                                    {{ character.name }}
                                    <Badge
                                        v-if="character.is_main"
                                        variant="secondary"
                                    >
                                        Main
                                    </Badge>
                                </p>
                                <div
                                    class="mt-0.5 flex items-center gap-2 text-xs text-muted-foreground"
                                >
                                    <span
                                        v-if="character.corporation"
                                        class="flex items-center gap-1"
                                    >
                                        <img
                                            :src="
                                                eveImage(
                                                    'corporation',
                                                    character.corporation.id,
                                                )!
                                            "
                                            :alt="
                                                character.corporation.name ?? ''
                                            "
                                            class="h-4 w-4 rounded-sm"
                                        />
                                        {{ character.corporation.name }}
                                    </span>
                                    <span
                                        v-if="character.alliance"
                                        class="flex items-center gap-1"
                                    >
                                        <img
                                            :src="
                                                eveImage(
                                                    'alliance',
                                                    character.alliance.id,
                                                )!
                                            "
                                            :alt="character.alliance.name ?? ''"
                                            class="h-4 w-4 rounded-sm"
                                        />
                                        {{ character.alliance.name }}
                                    </span>
                                </div>
                            </div>

                            <div class="flex items-center gap-3">
                                <Button
                                    v-if="!character.is_main"
                                    type="button"
                                    variant="ghost"
                                    size="icon"
                                    class="h-7 w-7 text-muted-foreground"
                                    title="Set as main character"
                                    :aria-label="`Set ${character.name} as your main character`"
                                    @click="setMainCharacter(character)"
                                >
                                    <Star class="h-4 w-4" />
                                </Button>
                                <span class="text-xs text-muted-foreground">
                                    <template v-if="character.inherits_source">
                                        Inherits the source
                                    </template>
                                    <template
                                        v-else-if="
                                            character.should_sync &&
                                            character.has_write_scope
                                        "
                                    >
                                        {{ character.synced_contacts_count }}
                                        synced
                                    </template>
                                </span>
                                <Badge
                                    v-if="character.inherits_source"
                                    variant="outline"
                                >
                                    Inherited
                                </Badge>
                                <a
                                    v-else-if="!character.has_write_scope"
                                    :href="syncScopesUrl"
                                    class="inline-flex items-center gap-1 text-xs font-medium text-amber-600 hover:underline dark:text-amber-400"
                                    :title="`Grant ${character.name} the contact permissions needed to sync standings`"
                                >
                                    <KeyRound class="h-3 w-3" />
                                    Grant sync access
                                </a>
                                <Switch
                                    v-else
                                    :model-value="character.should_sync"
                                    :aria-label="`Toggle standings sync for ${character.name}`"
                                    @update:model-value="
                                        (value) =>
                                            toggleSync(
                                                character,
                                                value === true,
                                            )
                                    "
                                />
                            </div>
                        </li>
                    </ul>
                </CardContent>
            </Card>
        </div>
    </AppLayout>
</template>
