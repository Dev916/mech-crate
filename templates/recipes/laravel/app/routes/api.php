<?php

use App\Http\Controllers\Api\HealthController;
use Illuminate\Support\Facades\Route;

/*
|--------------------------------------------------------------------------
| API Routes
|--------------------------------------------------------------------------
*/

Route::get('/health', HealthController::class)->name('api.health');

// Route::middleware('auth:sanctum')->group(function () {
//     Route::get('/user', fn () => auth()->user());
// });
