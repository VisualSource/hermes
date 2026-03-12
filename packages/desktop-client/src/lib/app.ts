import { NoiseSuppressor } from "./noise-suppressor";
import { SocketManager } from "./socket";
import { VoiceChannelRequest, VoiceChannelEventType, RtcNewCandidate } from "./proto/hermes";
import { RTC } from "./rtc";
export class App extends EventTarget {
	private static INSTANCE: App | null = null;
	public static get(): App {
		if (!App.INSTANCE) throw new Error("failed to get app instance");

		return App.INSTANCE;
	}

	static async create() {
		await NoiseSuppressor.create();
		await SocketManager.create();

		App.INSTANCE = new App();
	}

	private socket = SocketManager.get();
	private inVoice: boolean = false;
	private rtc = new RTC();

	constructor(){
		super();

	
		this.rtc.addEventListener("rtc-new-ice-candidate",(ev)=>{
			this.socket.send({ 
				rtcIce: RtcNewCandidate.create({
					target: ev.peerId,
					candidate: ev.candidate.toJSON().
				})
			});
		});

	}

	public joinVoice = (channelId: string) => {
		console.log("Join Voice channel", channelId);

		this.socket.send({
			voiceChannelRequest: VoiceChannelRequest.create({ 
				channelId, type: VoiceChannelEventType.Join 
			})
		})
	
		this.inVoice = true;

		this.dispatchEvent(new Event("voice-state-change"));
	};

	public leaveVoice = (channelId: string) => {
		console.log("Leaving voice channel",channelId);
		
		this.socket.send({
			voiceChannelRequest: VoiceChannelRequest.create({
				channelId, 
				type: VoiceChannelEventType.Leave
			})
		})

		this.inVoice = false;
		this.dispatchEvent(new Event("voice-state-change"));
	};

	public getProp = (prop: string) => {
		switch (prop) {
			case "inVoice":
				return this.inVoice;
			default:
				return null;
		}
	};
}

export const makeSubscription = <T = null>(event: string, prop: string) => {
	return {
		subscription: (callback: () => void) => {
			const inst = App.get();
			inst.addEventListener(event, callback);
			return () => {
				inst.removeEventListener(event, callback);
			};
		},
		snapshot: () => App.get().getProp(prop) as T,
	};
};
