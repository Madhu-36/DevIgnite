import React, { useEffect } from 'react';
import { CheckCircle, AlertCircle, Info, X } from 'lucide-react';
import { ToastMessage } from '../types';

interface ToastProps {
  toasts: ToastMessage[];
  onDismiss: (id: string) => void;
}

export const ToastContainer: React.FC<ToastProps> = ({ toasts, onDismiss }) => {
  return (
    <div className="fixed bottom-5 right-5 z-50 flex flex-col space-y-2 pointer-events-none">
      {toasts.map((toast) => (
        <ToastItem key={toast.id} toast={toast} onDismiss={onDismiss} />
      ))}
    </div>
  );
};

const ToastItem: React.FC<{ toast: ToastMessage; onDismiss: (id: string) => void }> = ({
  toast,
  onDismiss,
}) => {
  useEffect(() => {
    const timer = setTimeout(() => {
      onDismiss(toast.id);
    }, 4000);
    return () => clearTimeout(timer);
  }, [toast.id, onDismiss]);

  const icons = {
    success: <CheckCircle className="text-green-400 shrink-0" size={18} />,
    error: <AlertCircle className="text-red-400 shrink-0" size={18} />,
    info: <Info className="text-blue-400 shrink-0" size={18} />,
  };

  const borders = {
    success: 'border-green-800 bg-gray-900/90 text-green-200',
    error: 'border-red-800 bg-gray-900/90 text-red-200',
    info: 'border-blue-800 bg-gray-900/90 text-blue-200',
  };

  return (
    <div
      className={`pointer-events-auto flex items-center justify-between p-3 rounded-lg border shadow-lg backdrop-blur-md min-w-[280px] max-w-sm transition-all animate-slideIn ${
        borders[toast.type]
      }`}
    >
      <div className="flex items-center space-x-2 mr-3">
        {icons[toast.type]}
        <p className="text-xs font-medium text-white">{toast.message}</p>
      </div>
      <button
        onClick={() => onDismiss(toast.id)}
        className="text-gray-400 hover:text-white transition-colors p-1"
      >
        <X size={14} />
      </button>
    </div>
  );
};
