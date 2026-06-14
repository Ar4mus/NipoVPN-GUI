import { Store } from '@tauri-apps/plugin-store';
import { logMessage } from '$lib/logger';

export type LogLevel = 'INFO' | 'TRACE' | 'DEBUG';

export const LOG_LEVELS = ['INFO', 'TRACE', 'DEBUG'] as const;

export const DEFAULT_LOG_LEVEL: LogLevel = 'INFO';

export function normalizeLogLevel(value: unknown): LogLevel {
	const normalized =
		typeof value === 'string' ? value.trim().toUpperCase() : String(value).toUpperCase();

	return LOG_LEVELS.includes(normalized as LogLevel) ? (normalized as LogLevel) : DEFAULT_LOG_LEVEL;
}

export interface ProfileConfig {
	token: string;
	protocol: string;
	fakeUrls: string;
	methods: string;
	endPoints: string;
	timeout: string;
	pullTimeout: string;
	tunnelEnable: boolean;
	connectionReuse: boolean;
	tlsEnable: boolean;
	tlsVerifyPeer: boolean;
	tlsCertFile: string;
	tlsKeyFile: string;
	tlsCaFile: string;
	logLevel: LogLevel;
	listenIp: string;
	listenPort: string;
	serverIp: string;
	serverPort: string;
	httpVersion: string;
	userAgent: string;
	bufferSize: string;
}

export interface Profile {
	id: string;
	name: string;
	config: ProfileConfig;
}

type RawProfile = {
	name?: unknown;
	config?: Record<string, unknown>;
};

function coerceImportedConfigValue(
	field: keyof ProfileConfig,
	value: unknown
): ProfileConfig[keyof ProfileConfig] {
	if (field === 'logLevel') {
		return normalizeLogLevel(value);
	}

	if (
		field === 'tunnelEnable' ||
		field === 'connectionReuse' ||
		field === 'tlsEnable' ||
		field === 'tlsVerifyPeer'
	) {
		if (typeof value === 'boolean') {
			return value;
		}
	}

	if (value === null) {
		return '';
	}

	return String(value);
}

type MutableProfileConfig = {
	[K in keyof ProfileConfig]?: ProfileConfig[K];
};

type ProfileConfigField = {
	key: keyof ProfileConfig;
	label: string;
};

const PROFILE_CONFIG_FIELDS: ProfileConfigField[] = [
	{ key: 'token', label: 'TOKEN' },
	{ key: 'protocol', label: 'PROTOCOL' },
	{ key: 'fakeUrls', label: 'FAKE URLS' },
	{ key: 'methods', label: 'METHODS' },
	{ key: 'endPoints', label: 'END POINTS' },
	{ key: 'timeout', label: 'TIMEOUT' },
	{ key: 'pullTimeout', label: 'PULL TIMEOUT' },
	{ key: 'tunnelEnable', label: 'TUNNEL ENABLE' },
	{ key: 'connectionReuse', label: 'CONNECTION REUSE' },
	{ key: 'tlsEnable', label: 'TLS ENABLE' },
	{ key: 'tlsVerifyPeer', label: 'TLS VERIFY PEER' },
	{ key: 'tlsCertFile', label: 'TLS CERT FILE' },
	{ key: 'tlsKeyFile', label: 'TLS KEY FILE' },
	{ key: 'tlsCaFile', label: 'TLS CA FILE' },
	{ key: 'logLevel', label: 'LOG LEVEL' },
	{ key: 'listenIp', label: 'LISTEN IP' },
	{ key: 'listenPort', label: 'LISTEN PORT' },
	{ key: 'serverIp', label: 'SERVER IP' },
	{ key: 'serverPort', label: 'SERVER PORT' },
	{ key: 'httpVersion', label: 'HTTP VERSION' },
	{ key: 'userAgent', label: 'USER AGENT' },
	{ key: 'bufferSize', label: 'BUFFER SIZE' }
];

class ProfilesStore {
	private tauriStore: Store | null = null;

	profiles = $state<Profile[]>([]);
	selectedProfileId = $state<string>('');
	isLoading = $state<boolean>(true);

	constructor() {
		if (typeof window !== 'undefined') {
			this.init();
		}
	}

	private async init() {
		try {
			this.tauriStore = await Store.load('profiles.json');
			const savedProfiles = await this.tauriStore.get<Profile[]>('profiles');
			const savedSelected = await this.tauriStore.get<string>('selectedProfileId');

			if (savedProfiles && Array.isArray(savedProfiles)) {
				this.profiles = savedProfiles;
			} else {
				this.profiles = [];
				await this.tauriStore.set('profiles', []);
				await this.tauriStore.save();
			}

			if (savedSelected && this.profiles.some((p) => p.id === savedSelected)) {
				this.selectedProfileId = savedSelected;
			} else if (this.profiles.length > 0) {
				this.selectedProfileId = this.profiles[0].id;
				await this.tauriStore.set('selectedProfileId', this.selectedProfileId);
				await this.tauriStore.save();
			} else {
				this.selectedProfileId = '';
			}
		} catch (e) {
			await logMessage('warn', 'ProfilesStore', `Failed to load profiles from Tauri store, falling back to localStorage: ${e instanceof Error ? e.message : String(e)}`);
			try {
				const savedProfilesStr = localStorage.getItem('profiles');
				const savedSelected = localStorage.getItem('selectedProfileId');

				if (savedProfilesStr) {
					this.profiles = JSON.parse(savedProfilesStr);
				} else {
					this.profiles = [];
					localStorage.setItem('profiles', JSON.stringify(this.profiles));
				}

				if (savedSelected && this.profiles.some((p) => p.id === savedSelected)) {
					this.selectedProfileId = savedSelected;
				} else if (this.profiles.length > 0) {
					this.selectedProfileId = this.profiles[0].id;
					localStorage.setItem('selectedProfileId', this.selectedProfileId);
				} else {
					this.selectedProfileId = '';
				}
			} catch (err) {
				await logMessage('error', 'ProfilesStore', `LocalStorage fallback failed: ${err instanceof Error ? err.message : String(err)}`);
				this.profiles = [];
				this.selectedProfileId = '';
			}
		} finally {
			this.isLoading = false;
		}
	}

	createDefaultProfile(name: string): Profile {
		return {
			id: Math.random().toString(36).substring(2, 9),
			name,
			config: {
				token: '',
				protocol: 'socks5',
				fakeUrls: '',
				methods: 'GET\nPOST\nPUT\nDELETE',
				endPoints: 'api\nlogin\nuser\nupdate',
				timeout: '10',
				pullTimeout: '50',
				tunnelEnable: false,
				connectionReuse: true,
				tlsEnable: false,
				tlsVerifyPeer: false,
				tlsCertFile: '',
				tlsKeyFile: '',
				tlsCaFile: '',
				logLevel: DEFAULT_LOG_LEVEL,
				listenIp: '127.0.0.1',
				listenPort: '1080',
				serverIp: '',
				serverPort: '',
				httpVersion: '1.1',
				userAgent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
				bufferSize: '65536'
			}
		};
	}

	async save() {
		if (this.tauriStore) {
			try {
				await this.tauriStore.set('profiles', $state.snapshot(this.profiles));
				await this.tauriStore.set('selectedProfileId', this.selectedProfileId);
				await this.tauriStore.save();
			} catch (e) {
				await logMessage('error', 'ProfilesStore', `Failed to save to Tauri store: ${e instanceof Error ? e.message : String(e)}`);
			}
		} else {
			try {
				localStorage.setItem('profiles', JSON.stringify($state.snapshot(this.profiles)));
				localStorage.setItem('selectedProfileId', this.selectedProfileId);
			} catch (e) {
				await logMessage('error', 'ProfilesStore', `Failed to save to localStorage: ${e instanceof Error ? e.message : String(e)}`);
			}
		}
	}

	async addProfile(name: string, config: Partial<ProfileConfig>) {
		const defaultConf = this.createDefaultProfile(name).config;
		const finalConfig = { ...defaultConf, ...config };
		const newProfile: Profile = {
			id: Math.random().toString(36).substring(2, 9),
			name,
			config: finalConfig
		};
		this.profiles.push(newProfile);
		if (this.profiles.length === 1 || !this.selectedProfileId) {
			this.selectedProfileId = newProfile.id;
		}
		await this.save();
		return newProfile;
	}

	async updateProfile(id: string, name: string, config: Partial<ProfileConfig>) {
		const index = this.profiles.findIndex((p) => p.id === id);
		if (index !== -1) {
			this.profiles[index].name = name;
			this.profiles[index].config = { ...this.profiles[index].config, ...config };
			await this.save();
		}
	}

	async deleteProfile(id: string) {
		this.profiles = this.profiles.filter((p) => p.id !== id);
		if (this.selectedProfileId === id) {
			if (this.profiles.length > 0) {
				this.selectedProfileId = this.profiles[0].id;
			} else {
				this.selectedProfileId = '';
			}
		}
		await this.save();
	}

	async selectProfile(id: string) {
		if (this.profiles.some((p) => p.id === id)) {
			this.selectedProfileId = id;
			await this.save();
		}
	}

	async importProfile(inputString: string) {
		try {
			let cleaned = inputString.trim();
			if (cleaned.startsWith('nipovpn://')) {
				cleaned = cleaned.slice('nipovpn://'.length);
			}

			const decoded = atob(cleaned);
			const bytes = new Uint8Array(decoded.length);
			for (let i = 0; i < decoded.length; i++) {
				bytes[i] = decoded.charCodeAt(i);
			}
			const jsonText = new TextDecoder().decode(bytes);
			const parsed = JSON.parse(jsonText);

			if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
				throw new Error('Invalid profile structure: not an object');
			}

			const rawProfile = parsed as RawProfile;
			const name =
				typeof rawProfile.name === 'string' && rawProfile.name.trim()
					? rawProfile.name
					: 'Imported Profile';
			const rawConfig =
				rawProfile.config &&
				typeof rawProfile.config === 'object' &&
				!Array.isArray(rawProfile.config)
					? rawProfile.config
					: {};

			const importedConfig: MutableProfileConfig = {};
			const missingFields = PROFILE_CONFIG_FIELDS
				.filter(({ key }) => rawConfig[key] === undefined)
				.map(({ label }) => label);

			if (missingFields.length > 0) {
				await logMessage(
					'warn',
					'ProfilesStore',
					`Imported profile is missing fields: ${missingFields.join(', ')}. If connection fails, manually enter correct values.`
				);
			}

			const fields = PROFILE_CONFIG_FIELDS.map(({ key }) => key);

			for (const field of fields) {
				const value = rawConfig[field];
				if (value !== undefined) {
					Object.assign(importedConfig, {
						[field]: coerceImportedConfigValue(field, value)
					});
				}
			}

			importedConfig.logLevel = normalizeLogLevel(rawConfig.logLevel);

			const newProfile = await this.addProfile(name, importedConfig);
			await this.selectProfile(newProfile.id);
			return { profile: newProfile, warnings: missingFields };
		} catch (e: unknown) {
			await logMessage('error', 'ProfilesStore', `Failed to import profile: ${e instanceof Error ? e.message : String(e)}`);
			throw new Error('Invalid profile code', { cause: e });
		}
	}

	exportProfile(profile: Profile): string {
		try {
			const profileData = {
				name: profile.name,
				config: profile.config
			};

			const jsonString = JSON.stringify(profileData);

			const encoder = new TextEncoder();
			const bytes = encoder.encode(jsonString);

			const binary = String.fromCharCode.apply(null, Array.from(bytes));
			const base64 = btoa(binary);

			return `nipovpn://${base64}`;
		} catch (e: unknown) {
			void logMessage('error', 'ProfilesStore', `Failed to export profile: ${e instanceof Error ? e.message : String(e)}`);
			throw new Error('Failed to export profile', { cause: e });
		}
	}
}

export const profilesStore = new ProfilesStore();
