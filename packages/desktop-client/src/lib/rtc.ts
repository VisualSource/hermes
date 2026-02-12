//https://medium.com/swlh/manage-dynamic-multi-peer-connections-in-webrtc-3ff4e10f75b7
const DefaultRTCConfiguration: RTCConfiguration = {
    iceServers: [
        { urls: ["stun:stun.l.google.com:19302"] },
        { urls: ["stun:stun1.l.google.com:19302"] },
        { urls: ["stun:stun2.l.google.com:19302"] },
    ]
}

export class RTC extends EventTarget {
    private conns: Map<string,RTCPeerConnection> = new Map();

    /**
     * Init Rtc peer connetion and return session description that can be then set to peer
     *  
     * @example
     *  const desc = await rtc.initConnection(peerId);
     *  socket.send({ rtc: desc, peerId });
     * 
     * @param peerId 
     * @returns 
     */
    public async initConnection(peerId: string){
        const { reject, resolve, promise } = Promise.withResolvers<RTCSessionDescription>();

        const peer = new RTCPeerConnection(DefaultRTCConfiguration);

        peer.addEventListener("icecandidate",(ev)=>{
            if (ev.candidate) return;
            const description = peer.localDescription;
            if(!description) return reject("unable to get localDescription");
            resolve(description);
        });
        peer.addEventListener("icecandidateerror",(ev)=>reject(ev));
        this.initSharedEventHandler(peerId,peer);

        const description = await peer.createOffer({
            offerToReceiveAudio: true,
            offerToReceiveVideo: true,
        });
        await peer.setLocalDescription(description);

        this.conns.set(peerId,peer);

        return await promise;
    }

    /**
     * Init a RTC peer connection using a remoteDescription
     * 
     * @param peerId 
     * @param remoteDescription - offer from peer
     * @returns 
     */
    public async initConnectionFromRemote(peerId: string, remoteDescription: RTCSessionDescriptionInit){
        const peer = new RTCPeerConnection(DefaultRTCConfiguration);

        const { resolve, reject, promise } = Promise.withResolvers<RTCSessionDescription>();

        peer.addEventListener("icecandidate",(ev)=>{
            if(ev.candidate) return reject("got a candidate");
            const description = peer.localDescription;
            if(!description) return reject("unable to get local description");

            resolve(description);
        });
         peer.addEventListener("icecandidateerror",(ev)=>reject(ev));
        this.initSharedEventHandler(peerId,peer);
        
        await peer.setRemoteDescription(remoteDescription);

        const description = await peer.createAnswer();
        await peer.setLocalDescription(description);

        this.conns.set(peerId,peer);

        return await promise;
    }

    /**
     * finish setup of rtc connection from peer
     * 
     * @param peerId 
     * @param remoteDescription answer to an offer 
     */
    public async finishConnection(peerId: string, remoteDescription: RTCSessionDescription){
        const peer = this.conns.get(peerId);
        if(!peer) throw new Error(`unable to find peer with id of "${peerId}"`);

        await peer.setRemoteDescription(remoteDescription);
    }

    /**
     * Handle a negotation needed event from peer
     * 
     * @param peerId 
     * @param remoteDescription 
     * @returns 
     */
    public async handleNegotationNeeded(peerId: string, remoteDescription: RTCSessionDescription){
        const peer = this.conns.get(peerId);
        if(!peer) throw new Error(`unable to find peer with id of "${peerId}"`);

        await peer.setRemoteDescription(remoteDescription);

        const description = await peer.createAnswer();
        await peer.setLocalDescription(description);

        if(!peer.localDescription) throw new Error("unable to get local description");

        return peer.localDescription
    }

    private initSharedEventHandler(peerId: string, peer: RTCPeerConnection){
        peer.addEventListener("connectionstatechange",()=>{
            if(peer.connectionState === "closed") {
                this.conns.delete(peerId);
            }  

            this.dispatchEvent(new RTCConnectionStateChangeEvent(peerId,peer.connectionState));
        });
        peer.addEventListener("negotiationneeded",async ()=>{
             const offer = await peer.createOffer({ 
                offerToReceiveAudio: true,
                offerToReceiveVideo: true,
            }); 

            await peer.setLocalDescription(offer);
            if(!peer.localDescription) {
                this.dispatchEvent(new Event("rtc-negotation-start-failed"));
                return;
            }

            this.dispatchEvent(new RTCNegotationEvent(peerId,peer.localDescription));
        });
        peer.addEventListener("track",(ev)=>{
            this.dispatchEvent(new RTCTrackEvent(peerId,ev.track))
        });
    }
}

class RTCConnectionStateChangeEvent extends Event {
    constructor(public peerId: string, public state: RTCPeerConnectionState){
        super("rtc-connection-state-change");
    }
}
class RTCNegotationEvent extends Event {
    constructor(public peerId: string, public newDescription: RTCSessionDescription){
        super("rtc-negotation")
    }
}

class RTCTrackEvent extends Event {
    constructor(public peerId: string, public track: MediaStreamTrack ){
        super("rtc-track")
    }
}