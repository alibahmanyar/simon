export interface ApiResponse<T> {
	success: boolean;
	data: T | null;
	error: string | null;
}

export interface SystemCapabilities {
	cpu: boolean;
	memory: boolean;
	swap: boolean;
	load_average: boolean;
	network: boolean;
	disk: boolean;
	processes: boolean;
	docker: boolean;
	file_serving: boolean;
}

export interface HistoricalQueryOptions {
	resolution: 'second' | 'minute' | 'hour' | 'day';
	start_time?: number;
	end_time?: number;
	limit?: number;
}

export interface HistoricalSeries {
	cat: string;
	stype: string;
	name: string;
	timestamps: number[];
	values: number[];
}

export enum wsStatus {
	INIT = 0,
	WAITING = 1,
	CONNECTED = 2,
	DISCONNECTED = 3,
	ERROR = 4
}

export interface SystemData {
	t: number;
	sys: SystemInfo;
	mem: MemoryInfo;
	cpu: CPUInfo;
	net: NetworkInfo;
	disk: DiskInfo;
}

interface SystemInfo {
	name: string;
	kernel_ver: string;
	os_ver: string;
	os_name: string;
	host_name: string;
	load_avg: number[];
	uptime: number;
}

interface MemoryInfo {
	total_mem: number;
	used_mem: number;
	total_swap: number;
	used_swap: number;
}

interface CPUInfo {
	count: number;
	avg_usage: number;
	usage: number[];
}

interface NetworkInfo {
	interfaces: NetworkInterface[];
}

export interface NetworkInterface {
	name: string;
	rx: number;
	tx: number;
	receiveRate?: number;
	transmitRate?: number;
}

export interface NetworkStats {
	received: number;
	transmitted: number;
	time: number;
}

interface DiskInfo {
	disks: Disk[];
}

export interface Disk {
	fs: string;
	kind: string;
	total_space: number;
	free_space: number;
	mount_point: string;
	removable: boolean;
	io: number[];
}

export interface DockerPort {
	ip?: string;
	priv_port: number;
	pub_port?: number;
	protocol: string;
}

export interface DockerContainer {
	id: string;
	name: string;
	image: string;
	status: string;
	state: string;
	created: number;
	ports: DockerPort[];
	cpu_usage: number;
	mem_usage: number;
	mem_limit: number;
	net_io: [number, number];
	disk_io: [number, number];
}

export interface DockerInfo {
	t: number;
	containers: DockerContainer[];
}

export interface NotificationMethod {
	id: string;
	name: string;
	kind: string;
	enabled: boolean;
	config: {
		WebHook: {
			url: string;
			method: string;
			headers: Record<string, string>;
			body: string;
		};
	};
}

export interface AlertVar {
	cat: string; // Category
	var: string; // variable name (ex. rx_rate)
	resrc: string; // Resource name (ex. eth0)
}

export interface Alert {
	id: string;
	var: AlertVar;
	threshold: number;
	operator: string;
	enabled: boolean;
	time_window: number;
	firing: boolean;
	notif_methods: string[];
}

export interface FileEntry {
	name: string;
	is_dir: boolean;
	size: number;
	modified: number;
	created: number;
	permissions: string;
}

export interface DirectoryListing {
	path: string;
	entries: FileEntry[];
}
