@props(['tools' => []])

<div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8 relative isolate">
    <div class="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
        @foreach ($tools as $tool)
            <x-partials.home.tool-card
                :icon="$tool['icon']"
                :title="$tool['title']"
                :description="$tool['description']"
                :url="$tool['url']"
                :target="$tool['target']"
            />
        @endforeach
    </div>
</div>
