<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Spatie\MediaLibrary\HasMedia;
use Spatie\MediaLibrary\InteractsWithMedia;
use Spatie\MediaLibrary\MediaCollections\Models\Media;

class FullAsset extends Model implements HasMedia
{
    use InteractsWithMedia;

    /**
     * The attributes that aren't mass assignable.
     *
     * @var array
     */
    protected $guarded = [];

    /**
     * Registra las conversiones para imágenes de alta resolución.
     * Las dimensiones son mayores para cubrir pantallas 2K/4K y Retina.
     *
     * @param \Spatie\MediaLibrary\MediaCollections\Models\Media|null $media
     * @return void
     */
    public function registerMediaConversions(?Media $media = null): void
    {
        // --- Conversiones para AVIF ---
        $this->addMediaConversion('w-1600-avif')->width(1600)->format('avif')->nonQueued();
        $this->addMediaConversion('w-2400-avif')->width(2400)->format('avif')->nonQueued();
        $this->addMediaConversion('w-3200-avif')->width(3200)->format('avif')->nonQueued();

        // --- Conversiones para WEBP ---
        $this->addMediaConversion('w-1600-webp')->width(1600)->format('webp')->nonQueued();
        $this->addMediaConversion('w-2400-webp')->width(2400)->format('webp')->nonQueued();
        $this->addMediaConversion('w-3200-webp')->width(3200)->format('webp')->nonQueued();

        // --- Conversión de Fallback ---
        // JPG con una calidad ligeramente superior para mantener el detalle.
        $this->addMediaConversion('fallback-jpg')
             ->width(3200)
             ->format('jpg')
             ->quality(85) // Un poco más de calidad para imágenes destacadas.
             ->nonQueued(); // NOTA: En producción, es casi mandatorio usar colas para estas resoluciones.
    }
}
