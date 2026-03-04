import { ref, onUnmounted } from "vue";

export function useVoice(
  sendFn: (msg: Record<string, unknown>) => void,
  roomId: string,
  _userId: string
) {
  const peers = ref<Map<string, RTCPeerConnection>>(new Map());
  const localStream = ref<MediaStream | null>(null);
  const isMuted = ref(false);
  const isVoiceActive = ref(false);

  const config: RTCConfiguration = {
    iceServers: [
      { urls: "stun:stun.l.google.com:19302" },
      { urls: "stun:stun1.l.google.com:19302" },
    ],
  };

  const initLocalAudio = async () => {
    localStream.value = await navigator.mediaDevices.getUserMedia({
      audio: {
        echoCancellation: true,
        noiseSuppression: true,
        autoGainControl: true,
      },
      video: false,
    });
    isVoiceActive.value = true;
  };

  const createPeerConnection = (targetUserId: string): RTCPeerConnection => {
    const pc = new RTCPeerConnection(config);

    // Add local audio tracks
    if (localStream.value) {
      localStream.value.getTracks().forEach((track) => {
        pc.addTrack(track, localStream.value!);
      });
    }

    // Handle ICE candidates
    pc.onicecandidate = (event) => {
      if (event.candidate) {
        sendFn({
          type: "ice_candidate",
          room_id: roomId,
          target_user_id: targetUserId,
          candidate: JSON.stringify(event.candidate),
        });
      }
    };

    // Receive remote audio
    pc.ontrack = (event) => {
      const audio = new Audio();
      audio.srcObject = event.streams[0];
      audio.play().catch(console.error);
    };

    pc.onconnectionstatechange = () => {
      if (pc.connectionState === "failed" || pc.connectionState === "disconnected") {
        peers.value.delete(targetUserId);
        pc.close();
      }
    };

    peers.value.set(targetUserId, pc);
    return pc;
  };

  // Initiate call to a user
  const callUser = async (targetUserId: string) => {
    const pc = createPeerConnection(targetUserId);
    const offer = await pc.createOffer();
    await pc.setLocalDescription(offer);

    sendFn({
      type: "voice_offer",
      room_id: roomId,
      target_user_id: targetUserId,
      offer: JSON.stringify(offer),
    });
  };

  // Answer an incoming call
  const answerCall = async (
    targetUserId: string,
    offer: RTCSessionDescriptionInit
  ) => {
    const pc = createPeerConnection(targetUserId);
    await pc.setRemoteDescription(offer);
    const answer = await pc.createAnswer();
    await pc.setLocalDescription(answer);

    sendFn({
      type: "voice_answer",
      room_id: roomId,
      target_user_id: targetUserId,
      answer: JSON.stringify(answer),
    });
  };

  // Handle incoming answer
  const handleAnswer = async (
    targetUserId: string,
    answer: RTCSessionDescriptionInit
  ) => {
    const pc = peers.value.get(targetUserId);
    if (pc) {
      await pc.setRemoteDescription(answer);
    }
  };

  // Handle incoming ICE candidate
  const handleIceCandidate = async (
    targetUserId: string,
    candidate: RTCIceCandidateInit
  ) => {
    const pc = peers.value.get(targetUserId);
    if (pc) {
      await pc.addIceCandidate(candidate);
    }
  };

  // Toggle mute
  const toggleMute = () => {
    if (localStream.value) {
      localStream.value.getAudioTracks().forEach((track) => {
        track.enabled = isMuted.value; // Toggle: if muted, enable; if not, disable
      });
      isMuted.value = !isMuted.value;
    }
  };

  // Stop voice and clean up
  const stopVoice = () => {
    // Close all peer connections
    for (const [, pc] of peers.value) {
      pc.close();
    }
    peers.value.clear();

    // Stop local stream
    if (localStream.value) {
      localStream.value.getTracks().forEach((track) => track.stop());
      localStream.value = null;
    }
    isVoiceActive.value = false;
    isMuted.value = false;
  };

  // Remove a specific peer
  const removePeer = (targetUserId: string) => {
    const pc = peers.value.get(targetUserId);
    if (pc) {
      pc.close();
      peers.value.delete(targetUserId);
    }
  };

  onUnmounted(() => {
    stopVoice();
  });

  return {
    peers,
    localStream,
    isMuted,
    isVoiceActive,
    initLocalAudio,
    callUser,
    answerCall,
    handleAnswer,
    handleIceCandidate,
    toggleMute,
    stopVoice,
    removePeer,
  };
}