<script setup lang="ts">
import { index as overviewRoute } from '@/actions/App/Http/Controllers/Admin/AdministrationController';
import { update as updateDiscord } from '@/actions/App/Http/Controllers/Admin/DiscordSettingController';
import { edit as settingsRoute } from '@/actions/App/Http/Controllers/Admin/SettingsController';
import {
    sync as syncSource,
    update as updateSource,
} from '@/actions/App/Http/Controllers/Admin/StandingsSourceController';
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar';
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
    DialogFooter,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select';
import { countdown, dateWithAgo } from '@/lib/date';
import { eveImage } from '@/lib/eve';
import AdminLayout from '@/layouts/admin/Layout.vue';
import AppLayout from '@/layouts/AppLayout.vue';
import type { BreadcrumbItem } from '@/types';
import { Head, router, useForm } from '@inertiajs/vue3';
import { useIntervalFn, useNow } from '@vueuse/core';
import { Mail, RefreshCw } from '@lucide/vue';
import { computed, ref } from 'vue';

const REFRESH_INTERVAL_MS = 5 * 60 * 1000;

type SourceTypeOption = {
    value: string;
    label: string;
    entity_id: number | null;
    entity_name: string | null;
    has_scope: boolean;
    available: boolean;
};

const props = defineProps<{
    source: {
        type: string;
        entity_id: number;
        entity_name: string | null;
        last_synced_at: string | null;
    } | null;
    sourceTypes: SourceTypeOption[];
    adminCharacter: {
        id: number;
        name: string;
        has_mail_scope: boolean;
    } | null;
    contactsCount: number;
    discordSettings: { webhook_url: string | null; role_id: string | null };
    grantMailScopeUrl: string;
}>();

const breadcrumbs: BreadcrumbItem[] = [
    { title: 'Administration', href: overviewRoute() },
    { title: 'Settings', href: settingsRoute() },
];

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

const sourceForm = useForm<{ type: string }>({
    type: props.source?.type ?? '',
});

const discordForm = useForm({
    webhook_url: props.discordSettings.webhook_url ?? '',
    role_id: props.discordSettings.role_id ?? '',
});

const changeSourceOpen = ref(false);

function saveSource(): void {
    sourceForm.put(updateSource.url(), {
        preserveScroll: true,
        onSuccess: () => {
            changeSourceOpen.value = false;
        },
    });
}

function saveDiscord(): void {
    discordForm.put(updateDiscord.url(), { preserveScroll: true });
}

function syncNow(): void {
    router.post(syncSource.url(), {}, { preserveScroll: true });
}

function subjectImage(type: string, id: number): string {
    return eveImage(type, id) ?? '';
}
</script>

<template>
    <Head title="Admin settings" />

    <AppLayout :breadcrumbs="breadcrumbs">
        <AdminLayout
            title="Settings"
            description="Configure the standings source and notifications."
        >
            <div class="grid gap-6 lg:grid-cols-2">
                <!-- Source (read-only with confirm-to-change) -->
                <Card>
                    <CardHeader
                        class="flex flex-row items-start justify-between gap-4"
                    >
                        <div class="flex items-center gap-3">
                            <Avatar
                                v-if="source"
                                class="h-10 w-10 shrink-0 overflow-hidden rounded"
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
                                    class="rounded text-xs uppercase"
                                >
                                    {{ source.type.slice(0, 2) }}
                                </AvatarFallback>
                            </Avatar>
                            <div>
                                <CardTitle>Standings source</CardTitle>
                                <CardDescription>
                                    <template v-if="source">
                                        <span
                                            class="font-medium text-foreground"
                                            >{{
                                                source.entity_name ??
                                                source.entity_id
                                            }}</span
                                        >
                                        <span class="capitalize"
                                            >&middot; {{ source.type }}</span
                                        >
                                    </template>
                                    <template v-else>
                                        No source has been configured yet.
                                    </template>
                                </CardDescription>
                            </div>
                        </div>
                        <div class="flex shrink-0 items-center gap-2">
                            <Dialog v-model:open="changeSourceOpen">
                                <DialogTrigger as-child>
                                    <Button
                                        type="button"
                                        size="sm"
                                        :variant="
                                            source ? 'destructive' : 'default'
                                        "
                                    >
                                        {{
                                            source
                                                ? 'Change source'
                                                : 'Set source'
                                        }}
                                    </Button>
                                </DialogTrigger>
                                <DialogContent class="sm:max-w-md">
                                    <DialogHeader>
                                        <DialogTitle>
                                            {{
                                                source
                                                    ? 'Change standings source?'
                                                    : 'Set standings source'
                                            }}
                                        </DialogTitle>
                                        <DialogDescription>
                                            <template v-if="source">
                                                Changing the source clears the
                                                current standings and re-syncs
                                                every character.
                                            </template>
                                            <template v-else>
                                                Select which of your character,
                                                corporation or alliance defines
                                                the canonical standings.
                                            </template>
                                        </DialogDescription>
                                    </DialogHeader>
                                    <form
                                        class="space-y-4"
                                        @submit.prevent="saveSource"
                                    >
                                        <div class="space-y-2">
                                            <Label for="type">Source</Label>
                                            <Select v-model="sourceForm.type">
                                                <SelectTrigger
                                                    id="type"
                                                    class="w-full"
                                                >
                                                    <SelectValue
                                                        placeholder="Select your character, corp or alliance"
                                                    />
                                                </SelectTrigger>
                                                <SelectContent>
                                                    <SelectItem
                                                        v-for="type in sourceTypes"
                                                        :key="type.value"
                                                        :value="type.value"
                                                        :disabled="
                                                            !type.available
                                                        "
                                                    >
                                                        {{ type.label }}
                                                        <span
                                                            v-if="
                                                                type.entity_name
                                                            "
                                                            class="text-muted-foreground"
                                                        >
                                                            &mdash;
                                                            {{
                                                                type.entity_name
                                                            }}
                                                        </span>
                                                        <span
                                                            v-else-if="
                                                                !type.has_scope
                                                            "
                                                            class="text-muted-foreground"
                                                        >
                                                            (no permission)
                                                        </span>
                                                        <span
                                                            v-else
                                                            class="text-muted-foreground"
                                                        >
                                                            (unavailable)
                                                        </span>
                                                    </SelectItem>
                                                </SelectContent>
                                            </Select>
                                            <p
                                                v-if="sourceForm.errors.type"
                                                class="text-sm text-destructive"
                                            >
                                                {{ sourceForm.errors.type }}
                                            </p>
                                        </div>
                                        <DialogFooter>
                                            <DialogClose as-child>
                                                <Button
                                                    type="button"
                                                    variant="outline"
                                                >
                                                    Cancel
                                                </Button>
                                            </DialogClose>
                                            <Button
                                                type="submit"
                                                :disabled="
                                                    sourceForm.processing
                                                "
                                            >
                                                {{
                                                    source
                                                        ? 'Change source'
                                                        : 'Save source'
                                                }}
                                            </Button>
                                        </DialogFooter>
                                    </form>
                                </DialogContent>
                            </Dialog>
                            <Button
                                v-if="source"
                                type="button"
                                variant="secondary"
                                size="sm"
                                @click="syncNow"
                            >
                                <RefreshCw class="h-4 w-4" />
                                Sync now
                            </Button>
                        </div>
                    </CardHeader>
                    <CardContent>
                        <div
                            v-if="!adminCharacter"
                            class="mb-4 rounded-md border border-destructive/40 bg-destructive/10 px-4 py-3 text-sm text-destructive"
                        >
                            Your active character could not be resolved.
                        </div>

                        <dl
                            v-if="source"
                            class="grid gap-3 text-sm sm:grid-cols-3"
                        >
                            <div>
                                <dt class="text-muted-foreground">
                                    Canonical contacts
                                </dt>
                                <dd class="font-medium">{{ contactsCount }}</dd>
                            </div>
                            <div>
                                <dt class="text-muted-foreground">
                                    Last refreshed
                                </dt>
                                <dd class="font-medium">{{ lastRefreshed }}</dd>
                            </div>
                            <div v-if="nextRefresh">
                                <dt class="text-muted-foreground">
                                    Next refresh
                                </dt>
                                <dd class="font-medium tabular-nums">
                                    {{ nextRefresh }}
                                </dd>
                            </div>
                        </dl>
                        <p v-else class="text-sm text-muted-foreground">
                            No source is configured yet.
                        </p>

                        <div
                            v-if="
                                adminCharacter && !adminCharacter.has_mail_scope
                            "
                            class="mt-4 flex flex-wrap items-center justify-between gap-3 rounded-md border bg-muted/40 px-4 py-3"
                        >
                            <div class="flex items-start gap-3 text-sm">
                                <Mail
                                    class="mt-0.5 h-4 w-4 shrink-0 text-muted-foreground"
                                />
                                <p class="text-muted-foreground">
                                    <span class="font-medium text-foreground">
                                        In-game expiry mail is off.
                                    </span>
                                    Grant the mail permission so pilots whose
                                    tokens expired get an EVE mail asking them
                                    to re-authenticate.
                                </p>
                            </div>
                            <Button
                                as-child
                                type="button"
                                size="sm"
                                variant="outline"
                            >
                                <a :href="grantMailScopeUrl">
                                    Grant mail permission
                                </a>
                            </Button>
                        </div>
                        <p
                            v-else-if="adminCharacter"
                            class="mt-4 flex items-center gap-2 text-xs text-muted-foreground"
                        >
                            <Mail class="h-3.5 w-3.5" />
                            In-game expiry mail is enabled for
                            {{ adminCharacter.name }}.
                        </p>
                    </CardContent>
                </Card>

                <!-- Discord notifications -->
                <Card>
                    <CardHeader>
                        <CardTitle>Discord notifications</CardTitle>
                        <CardDescription>
                            Where standing requests are announced. Optionally
                            ping a role.
                        </CardDescription>
                    </CardHeader>
                    <CardContent>
                        <form class="space-y-4" @submit.prevent="saveDiscord">
                            <div class="grid gap-4 sm:grid-cols-[2fr_1fr]">
                                <div class="space-y-2">
                                    <Label for="webhook_url">Webhook URL</Label>
                                    <Input
                                        id="webhook_url"
                                        v-model="discordForm.webhook_url"
                                        type="url"
                                        placeholder="https://discord.com/api/webhooks/..."
                                    />
                                    <p
                                        v-if="discordForm.errors.webhook_url"
                                        class="text-sm text-destructive"
                                    >
                                        {{ discordForm.errors.webhook_url }}
                                    </p>
                                </div>
                                <div class="space-y-2">
                                    <Label for="role_id"
                                        >Role to ping (optional)</Label
                                    >
                                    <Input
                                        id="role_id"
                                        v-model="discordForm.role_id"
                                        placeholder="Role ID"
                                    />
                                    <p
                                        v-if="discordForm.errors.role_id"
                                        class="text-sm text-destructive"
                                    >
                                        {{ discordForm.errors.role_id }}
                                    </p>
                                </div>
                            </div>
                            <div class="flex justify-end">
                                <Button
                                    type="submit"
                                    :disabled="discordForm.processing"
                                >
                                    Save Discord settings
                                </Button>
                            </div>
                        </form>
                    </CardContent>
                </Card>
            </div>
        </AdminLayout>
    </AppLayout>
</template>
