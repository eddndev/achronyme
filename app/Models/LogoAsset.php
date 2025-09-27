<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Spatie\MediaLibrary\HasMedia;
use Spatie\MediaLibrary\InteractsWithMedia;
use Spatie\MediaLibrary\MediaCollections\Models\Media;
use Spatie\Image\Enums\Fit;

class LogoAsset extends Model implements HasMedia
{
    use InteractsWithMedia;

    /**
     * The attributes that aren't mass assignable.
     *
     * @var array
     */
    protected $guarded = [];

    /**
     * Registra las conversiones para logos e imágenes pequeñas.
     * Se usa PNG como fallback para mantener la transparencia.
     *
     * @param \Spatie\MediaLibrary\MediaCollections\Models\Media|null $media
     * @return void
     */
    public function registerMediaConversions(?Media $media = null): void
    {
        // --- Conversiones para AVIF y WEBP (soportan transparencia) ---
        $this->addMediaConversion('w-150-avif')->width(150)->format('avif')->nonQueued();
        $this->addMediaConversion('w-300-avif')->width(300)->format('avif')->nonQueued();
        $this->addMediaConversion('w-500-avif')->width(500)->format('avif')->nonQueued();

        $this->addMediaConversion('w-150-webp')->width(150)->format('webp')->nonQueued();
        $this->addMediaConversion('w-300-webp')->width(300)->format('webp')->nonQueued();
        $this->addMediaConversion('w-500-webp')->width(500)->format('webp')->nonQueued();

        // --- Conversión de Fallback ---
        // PNG para mantener la transparencia del logo.
        // Usamos fit('max') para no agrandar imágenes si son más pequeñas que el target.
        $this->addMediaConversion('fallback-png')
             ->fit(Fit::Max, 500, 500) // Se ajusta dentro de un contenedor de 500x500 sin recortar ni estirar.
             ->format('png')
             ->nonQueued();
    }
}
