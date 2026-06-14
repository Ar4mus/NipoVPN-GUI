import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import tailwindcss from '@tailwindcss/vite';
import adapter from '@sveltejs/adapter-static';

export default defineConfig({
    plugins: [
        sveltekit({
            compilerOptions: {
                runes: ({ filename }) =>
                    filename.split(/[/\\]/).includes('node_modules') ? undefined : true
            },
            adapter: adapter({
                pages: 'build',
                assets: 'build',
                fallback: 'index.html',
                precompress: false,
                strict: true
            })
        }),
        tailwindcss()
    ],
    server: {
        port: 5173,
        strictPort: true,
        watch: {
            ignored: ['**/src-tauri/target/**']
        }
    }
});