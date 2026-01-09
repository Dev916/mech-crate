<?php

use Illuminate\Support\Facades\Route;

Route::get('/status', function () {
    return response()->json([
        'status' => 'ok',
        'service' => '{{SERVICE_NAME}}',
        'timestamp' => now()->toIso8601String(),
    ]);
});
