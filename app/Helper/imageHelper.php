<?php

use App\Models\FullAsset;
use App\Models\MediumAsset;
use Illuminate\Support\Facades\Cache;
use App\Models\LogoAsset;
use Illuminate\Support\Str;

if (! function_exists('get_medium_asset')) {
    /**
     * Devuelve el modelo MediumAsset cacheado por original_path.
     */
    function get_medium_asset(string $originalPath): ?MediumAsset
    {
        $cacheKey = 'medium_asset:'.md5($originalPath); // evita colisiones por basename()
        return Cache::rememberForever($cacheKey, function () use ($originalPath) {
            return MediumAsset::where('original_path', $originalPath)->first();
        });
    }
}

if (! function_exists('get_image')) {
    /**
     * URL de una conversión puntual (ej: w=800, format=avif) o fallback.
     */
    function get_image(string $originalPath, array $options = []): string
    {
        $asset = get_medium_asset($originalPath);
        if (! $asset) return asset('images/placeholder.jpg');

        $collection = $options['collection'] ?? 'default';

        // Si no pasan opciones (o sólo collection), usa fallback directo
        if (empty(array_diff_key($options, array_flip(['collection'])))) {
            return $asset->getFirstMediaUrl($collection, 'fallback-jpg');
        }

        $format = $options['format'] ?? 'jpg';
        if ($format === 'jpg') {
            return $asset->getFirstMediaUrl($collection, 'fallback-jpg');
        }

        $w = (int)($options['w'] ?? 1200);
        $allowed = [400, 800, 1200];
        if (!in_array($w, $allowed, true)) {
            $w = 1200;
        }

        return $asset->getFirstMediaUrl($collection, "w-{$w}-{$format}");
    }

}

if (! function_exists('get_medium_sources')) {
    /**
     * Devuelve un array ['avif' => '...srcset', 'webp' => '...srcset', 'fallback' => 'url'].
     */
    function get_medium_sources(string $originalPath, array $widths = [400,800,1200], string $collection = 'default'): array
    {
        $sources = [
            'avif'    => '',
            'webp'    => '',
            'fallback'=> get_image($originalPath, ['collection' => $collection]) // jpg 1200
        ];

        foreach (['avif','webp'] as $format) {
            $sets = [];
            foreach ($widths as $w) {
                $sets[] = get_image($originalPath, ['w' => $w, 'format' => $format, 'collection' => $collection]) . " {$w}w";
            }
            $sources[$format] = implode(', ', $sets);
        }

        return $sources;
    }
}

if (! function_exists('get_logo_asset')) {
    function get_logo_asset(string $originalPath): ?LogoAsset
    {
        $key = 'logo_asset:'.md5($originalPath);
        return Cache::rememberForever($key, fn() =>
            LogoAsset::where('original_path', $originalPath)->first()
        );
    }
}

if (! function_exists('get_logo_image')) {
    /**
     * Devuelve URL de conversión de LogoAsset (o fallback si no existe).
     * $options = ['w' => 150|300|500, 'format' => 'avif'|'webp'|'png', 'collection' => 'default']
     */
    function get_logo_image(string $originalPath, array $options = []): string
    {
        $asset = get_logo_asset($originalPath);
        if (! $asset) return asset('images/placeholder-logo.png');

        $collection = $options['collection'] ?? 'default';
        $format = $options['format'] ?? 'png';

        // Para PNG no necesitamos width: salimos antes
        if ($format === 'png') {
            return $asset->getFirstMediaUrl($collection, 'fallback-png');
        }

        // Sanitiza width una sola vez
        $w = (int)($options['w'] ?? 300);
        $allowed = [150, 300, 500];
        if (!in_array($w, $allowed, true)) {
            $w = 300;
        }

        $map = [
            'avif' => [150 => 'w-150-avif', 300 => 'w-300-avif', 500 => 'w-500-avif'],
            'webp' => [150 => 'w-150-webp', 300 => 'w-300-webp', 500 => 'w-500-webp'],
        ];

        return $asset->getFirstMediaUrl($collection, $map[$format][$w]);
    }

}

if (! function_exists('get_full_asset')) {
    /**
     * Obtiene el modelo FullAsset cacheado por original_path.
     */
    function get_full_asset(string $originalPath): ?FullAsset
    {
        $key = 'full_asset:'.md5($originalPath);
        return Cache::rememberForever($key, fn () =>
            FullAsset::where('original_path', $originalPath)->first()
        );
    }
}

if (! function_exists('get_full_image')) {
    /**
     * Devuelve la URL de una conversión puntual de FullAsset o fallback.
     *
     * $options = [
     *   'w'          => 1600|2400|3200,
     *   'format'     => 'avif'|'webp'|'jpg',
     *   'collection' => 'default'
     * ]
     */
    function get_full_image(string $originalPath, array $options = []): string
    {
        $asset = get_full_asset($originalPath);
        if (! $asset) return asset('images/placeholder-large.jpg');

        $collection = $options['collection'] ?? 'default';

        $format = $options['format'] ?? 'jpg';
        if ($format === 'jpg') {
            return $asset->getFirstMediaUrl($collection, 'fallback-jpg');
        }

        $w = (int)($options['w'] ?? 3200);
        $allowed = [1600, 2400, 3200];
        if (!in_array($w, $allowed, true)) {
            $w = 3200;
        }

        return $asset->getFirstMediaUrl($collection, "w-{$w}-{$format}");
    }

}

if (! function_exists('get_full_sources')) {
    /**
     * Devuelve:
     *  [
     *    'avif'     => 'url1600 1600w, url2400 2400w, url3200 3200w',
     *    'webp'     => '...',
     *    'fallback' => 'url-jpg-3200'
     *  ]
     */
    function get_full_sources(string $originalPath, array $widths = [1600,2400,3200], string $collection = 'default'): array
    {
        $widths = array_values(array_intersect($widths, [1600,2400,3200])); // sanea entrada
        sort($widths);

        $makeSet = function (string $format) use ($originalPath, $widths, $collection): string {
            return collect($widths)->map(function ($w) use ($format, $originalPath, $collection) {
                return get_full_image($originalPath, ['w' => $w, 'format' => $format, 'collection' => $collection]) . " {$w}w";
            })->implode(', ');
        };

        return [
            'avif'     => $makeSet('avif'),
            'webp'     => $makeSet('webp'),
            'fallback' => get_full_image($originalPath, ['format' => 'jpg', 'collection' => $collection]),
        ];
    }
}