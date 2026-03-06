import { useState, useCallback, useEffect, useRef } from "react";
import { useNavigate } from "react-router-dom";
import { QrCode, Smartphone, Sprout, RefreshCw } from "lucide-react";
import { clsx } from "clsx";
import { Button } from "../components/Button";
import * as api from "../api";

type LoginMethod = "qr" | "mp";

export default function LoginPage() {
  const navigate = useNavigate();
  const [method, setMethod] = useState<LoginMethod>("qr");
  const [qrImage, setQrImage] = useState<string | null>(null);
  const [mpCode, setMpCode] = useState<string | null>(null);
  const [statusMsg, setStatusMsg] = useState("");
  const [loading, setLoading] = useState(false);
  const [connecting, setConnecting] = useState(false);
  const pollRef = useRef<ReturnType<typeof setInterval>>(undefined);

  const stopPolling = useCallback(() => {
    if (pollRef.current) {
      clearInterval(pollRef.current);
      pollRef.current = undefined;
    }
  }, []);

  useEffect(() => () => stopPolling(), [stopPolling]);

  const requestQr = useCallback(async () => {
    setLoading(true);
    setStatusMsg("");
    stopPolling();
    try {
      const res = await api.requestQrCode("vip");
      setQrImage(`data:image/png;base64,${res.qrcode}`);
      setStatusMsg("请使用 QQ 扫描二维码");

      pollRef.current = setInterval(async () => {
        try {
          const status = await api.checkQrStatus(res.qrsig, "vip");
          if (status.ret === "0") {
            stopPolling();
            setStatusMsg(`欢迎, ${status.nickname}! 正在获取授权码...`);
            const mpRes = await api.requestMpLoginCode();
            setConnecting(true);
            setStatusMsg("正在连接游戏服务器...");
            await api.connectAndLogin(mpRes.code);
            navigate("/");
          } else if (status.ret === "65") {
            setStatusMsg("二维码已过期，请刷新");
            stopPolling();
          } else if (status.ret === "67") {
            setStatusMsg("请在手机上确认...");
          } else if (status.ret === "66") {
            setStatusMsg("等待扫码...");
          }
        } catch {
          // keep polling
        }
      }, 2000);
    } catch (e) {
      setStatusMsg(`失败: ${e}`);
    } finally {
      setLoading(false);
    }
  }, [stopPolling, navigate]);

  const requestMp = useCallback(async () => {
    setLoading(true);
    setStatusMsg("");
    stopPolling();
    try {
      const res = await api.requestMpLoginCode();
      setMpCode(res.code);
      setQrImage(`data:image/png;base64,${res.qrcode}`);
      setStatusMsg("请使用微信扫码授权");
    } catch (e) {
      setStatusMsg(`失败: ${e}`);
    } finally {
      setLoading(false);
    }
  }, [stopPolling]);

  const connectWithCode = useCallback(
    async (code: string) => {
      setConnecting(true);
      setStatusMsg("正在连接游戏服务器...");
      try {
        await api.connectAndLogin(code);
        navigate("/");
      } catch (e) {
        setStatusMsg(`连接失败: ${e}`);
        setConnecting(false);
      }
    },
    [navigate]
  );

  const handleMethodSwitch = (m: LoginMethod) => {
    stopPolling();
    setMethod(m);
    setQrImage(null);
    setMpCode(null);
    setStatusMsg("");
  };

  return (
    <div className="flex min-h-screen items-center justify-center bg-surface-dim p-4">
      <div className="w-full max-w-sm space-y-6">
        <div className="text-center">
          <Sprout className="mx-auto size-12 text-primary-500" />
          <h1 className="mt-3 text-2xl font-bold">Farm Pilot</h1>
          <p className="mt-1 text-sm text-on-surface-muted">
            登录以开始自动化农场
          </p>
        </div>

        <div className="flex rounded-lg bg-surface-bright p-1">
          {(
            [
              { key: "qr", icon: QrCode, label: "QQ 扫码" },
              { key: "mp", icon: Smartphone, label: "微信小程序" },
            ] as const
          ).map(({ key, icon: Icon, label }) => (
            <button
              key={key}
              onClick={() => handleMethodSwitch(key)}
              className={clsx(
                "flex flex-1 items-center justify-center gap-2 rounded-md py-2 text-sm font-medium transition-colors",
                method === key
                  ? "bg-surface text-on-surface shadow-sm"
                  : "text-on-surface-muted hover:text-on-surface"
              )}
            >
              <Icon className="size-4" />
              {label}
            </button>
          ))}
        </div>

        <div className="rounded-card border border-border bg-surface p-6">
          {qrImage ? (
            <div className="flex flex-col items-center gap-4">
              <div className="rounded-xl border border-border bg-white p-3">
                <img
                  src={qrImage}
                  alt="二维码"
                  className="size-48 object-contain"
                />
              </div>
              <Button
                variant="ghost"
                size="sm"
                icon={<RefreshCw className="size-3.5" />}
                onClick={method === "qr" ? requestQr : requestMp}
                loading={loading}
              >
                刷新
              </Button>
            </div>
          ) : (
            <div className="flex flex-col items-center gap-4 py-6">
              <div className="rounded-full bg-surface-bright p-4">
                {method === "qr" ? (
                  <QrCode className="size-8 text-on-surface-muted" />
                ) : (
                  <Smartphone className="size-8 text-on-surface-muted" />
                )}
              </div>
              <Button
                onClick={method === "qr" ? requestQr : requestMp}
                loading={loading}
              >
                {method === "qr" ? "获取二维码" : "获取授权码"}
              </Button>
            </div>
          )}

          {method === "mp" && mpCode && (
            <div className="mt-4 space-y-3">
              <div className="rounded-lg bg-surface-bright px-3 py-2">
                <p className="text-xs text-on-surface-muted mb-1">授权码</p>
                <p className="font-mono text-sm select-all break-all">
                  {mpCode}
                </p>
              </div>
              <Button
                className="w-full"
                onClick={() => connectWithCode(mpCode)}
                loading={connecting}
              >
                连接
              </Button>
            </div>
          )}
        </div>

        {statusMsg && (
          <p className="text-center text-sm text-on-surface-muted">
            {statusMsg}
          </p>
        )}
      </div>
    </div>
  );
}
