<script lang="ts">
	import logo from '$lib/assets/icon.png';
</script>

<div class="flex items-center gap-3">
	<img src={logo} alt="Logo" class="w-14 h-14 object-contain" />

	<div class="flex flex-col justify-center">
		<div class="flex items-center gap-2 ">
			<span class="text-2xl font-semibold tracking-tight text-slate-900 dark:text-white"
				>NipoVPN GUI</span
			>
			<span class="mt-1 text-[10px] font-medium bg-primary-300-700 px-1.5 py-0.5 rounded-base"
				>v0.2.1 beta</span
			>
		</div>

		<span class="text-[10px] text-slate-500 dark:text-slate-400">based on nipovpn v1.1.59</span>
	</div>
</div>
