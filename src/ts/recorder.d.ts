
declare type RecorderEventType = "streamError" | "streamReady" | "dataAvailable" | "start" | "pause" | "resume" | "stop";
interface RecordingDataAvailableEvent {
	detail: Uint8Array,
}

declare class Recorder {
	constructor(config?: { encoderPath: string, leaveStreamOpen: boolean });
	initStream();
	start();
	stop();
	clearStream();
	addEventListener( type: RecorderEventType, listener: (ev) => void, useCapture? );
	static isRecordingSupported(): boolean;
	audioContext: any;
}
