const API_BASE = import.meta.env.VITE_API_BASE || '/api'

export const runLoadTest = async (config) => {
  const response = await fetch(`${API_BASE}/run`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(config),
  });

  if (!response.ok) {
    const data = await response.json().catch(() => ({}));
    throw new Error(data.error || 'Failed to start load test');
  }

  return response.json();
};
