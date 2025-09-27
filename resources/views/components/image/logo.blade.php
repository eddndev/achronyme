@props([
  'src' => '',                 {{-- original_path (ej: 'resources/images/brand/logo.png') --}}
  'alt' => '',
  'size' => 'md',              {{-- sm|md|lg -> 150|300|500 px --}}
  'class' => 'inline-block h-auto',  {{-- no responsivo, pero estilizable --}}
  'loading' => 'lazy',
  'priority' => false,         {{-- true => eager + fetchpriority=high --}}
  'collection' => 'default',   {{-- si usas otra colección en Spatie --}}
  'width' => null,             {{-- opcional: fuerza width para CLS --}}
  'height' => null,            {{-- opcional: fuerza height para CLS --}}
  'density' => true,           {{-- genera 1x/2x si hay conversión superior --}}
])

@php
  $map = ['sm' => 150, 'md' => 300, 'lg' => 500];
  $baseW = $map[$size] ?? 300;

  // 1x y 2x (si hay conversión disponible superior)
  $w1x = $baseW;
  $w2x = match($baseW) {
    150 => 300,
    300 => 500,
    500 => 500, // no hay más grande, mantenemos 500
    default => 300
  };

  // Utiliza helpers si existen, si no, placeholder.
  $hasHelper = function_exists('get_logo_image');

  echo $hasHelper ? null : "helper not found. Using placeholder image.\n";

  $avif1x = $hasHelper ? get_logo_image($src, ['w' => $w1x, 'format' => 'avif', 'collection' => $collection]) : asset('images/placeholder-logo.png');
  $webp1x = $hasHelper ? get_logo_image($src, ['w' => $w1x, 'format' => 'webp', 'collection' => $collection]) : asset('images/placeholder-logo.png');
  $pngFallback = $hasHelper ? get_logo_image($src, ['format' => 'png', 'collection' => $collection]) : asset('images/placeholder-logo.png');

  // 2x
  $avif2x = $hasHelper ? get_logo_image($src, ['w' => $w2x, 'format' => 'avif', 'collection' => $collection]) : $avif1x;
  $webp2x = $hasHelper ? get_logo_image($src, ['w' => $w2x, 'format' => 'webp', 'collection' => $collection]) : $webp1x;

  $srcsetAvif = $density ? "{$avif1x} 1x, {$avif2x} 2x" : $avif1x;
  $srcsetWebp = $density ? "{$webp1x} 1x, {$webp2x} 2x" : $webp1x;

  // Si no pasas width/height, usa el baseW como ancho sugerido (altura flexible).
  $finalWidth  = $width  ?? $baseW;
  $finalHeight = $height ?? null;


@endphp

<picture {{ $attributes->merge(['class' => 'inline-block']) }}>
  <source type="image/avif" srcset="{{ $srcsetAvif }}">
  <source type="image/webp" srcset="{{ $srcsetWebp }}">
  <img
    src="{{ $pngFallback }}"
    alt="{{ $alt }}"
    class="{{ $class }}"
    @if($finalWidth)  width="{{ $finalWidth }}" @endif
    @if($finalHeight) height="{{ $finalHeight }}" @endif
    loading="{{ $priority ? 'eager' : $loading }}"
    decoding="async"
    @if($priority) fetchpriority="high" @endif
  />
</picture>
