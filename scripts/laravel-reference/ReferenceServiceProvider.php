<?php

// Copied into the reference checkout by run.sh (never part of main).
//
// - GET /_reference/login/{user}?character={id}: signs in as a user (the
//   app only has EVE SSO), with the active character in the session when
//   given, and answers 204. record.py keeps the session cookie.
// - Every outgoing HTTP request is answered from
//   storage/app/http-fakes.json (`METHOD url` or `url` keys, record.py
//   writes them per scenario) and logged to storage/logs/http.jsonl.
//   Unknown URLs answer 404.

declare(strict_types=1);

namespace App\Providers;

use App\Models\User;
use Illuminate\Http\Client\Request;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Facades\Route;
use Illuminate\Support\ServiceProvider;

final class ReferenceServiceProvider extends ServiceProvider
{
    public function boot(): void
    {
        Route::middleware('web')->get('/_reference/login/{user}', function (string $user) {
            $model = User::query()->findOrFail((int) $user);
            Auth::login($model, remember: true);
            if (request()->filled('character')) {
                $model->setActiveCharacter((int) request('character'));
            }

            return response()->noContent();
        });

        $path = storage_path('app/http-fakes.json');
        $fakes = is_file($path) ? json_decode((string) file_get_contents($path), true) : [];

        Http::fake(function (Request $request) use ($fakes) {
            file_put_contents(storage_path('logs/http.jsonl'), json_encode([
                'method' => $request->method(),
                'url' => $request->url(),
                'headers' => array_map(fn ($values) => $values[0] ?? null, $request->headers()),
                'body' => $request->body(),
            ])."\n", FILE_APPEND);

            $fake = $fakes[$request->method().' '.$request->url()] ?? $fakes[$request->url()] ?? null;

            if ($fake === null) {
                return Http::response(null, 404);
            }

            return Http::response($fake['body'] ?? null, $fake['status'] ?? 200, $fake['headers'] ?? []);
        });
    }
}
