#!/usr/bin/env python3
"""Simple HTTP server for research dashboard"""
import json
import duckdb
from http.server import HTTPServer, SimpleHTTPRequestHandler
from urllib.parse import parse_qs, urlparse
import os

DB_PATH = "../data/research.duckdb"

class DashboardHandler(SimpleHTTPRequestHandler):
    def do_GET(self):
        parsed = urlparse(self.path)

        if parsed.path == '/api/stats':
            self.serve_stats()
        elif parsed.path == '/api/positions':
            self.serve_positions()
        elif parsed.path == '/api/recent-trades':
            self.serve_recent_trades()
        else:
            # Serve static files
            super().do_GET()

    def serve_stats(self):
        """Get overall statistics"""
        try:
            conn = duckdb.connect(DB_PATH, read_only=True)

            # Overall stats
            stats = conn.execute("""
                SELECT
                    (SELECT COUNT(*) FROM trades) as total_trades,
                    (SELECT COUNT(*) FROM positions WHERE exit_time_micros IS NULL) as open_positions,
                    (SELECT COUNT(*) FROM positions WHERE exit_time_micros IS NOT NULL) as closed_positions,
                    (SELECT COUNT(*) FILTER (WHERE pnl_sol > 0) FROM positions WHERE exit_time_micros IS NOT NULL) as wins,
                    (SELECT COUNT(*) FILTER (WHERE pnl_sol <= 0) FROM positions WHERE exit_time_micros IS NOT NULL) as losses,
                    (SELECT COALESCE(SUM(pnl_sol), 0) FROM positions WHERE exit_time_micros IS NOT NULL) as total_pnl_sol,
                    (SELECT COALESCE(AVG(pnl_percent), 0) FROM positions WHERE exit_time_micros IS NOT NULL) as avg_pnl_pct,
                    (SELECT COALESCE(AVG(hold_duration_secs), 0) FROM positions WHERE exit_time_micros IS NOT NULL) as avg_hold_secs,
                    (SELECT COALESCE(AVG(holder_count_entry), 0) FROM positions WHERE exit_time_micros IS NOT NULL) as avg_entry_holders,
                    (SELECT COALESCE(AVG(holder_count_exit), 0) FROM positions WHERE exit_time_micros IS NOT NULL) as avg_exit_holders
            """).fetchone()

            closed = stats[2] or 0
            wins = stats[3] or 0

            result = {
                'total_trades': stats[0] or 0,
                'open_positions': stats[1] or 0,
                'closed_positions': closed,
                'wins': wins,
                'losses': stats[4] or 0,
                'win_rate_pct': (wins / closed * 100) if closed > 0 else 0,
                'total_pnl_sol': float(stats[5]) if stats[5] else 0,
                'avg_pnl_pct': float(stats[6]) if stats[6] else 0,
                'avg_hold_secs': float(stats[7]) if stats[7] else 0,
                'avg_entry_holders': float(stats[8]) if stats[8] else 0,
                'avg_exit_holders': float(stats[9]) if stats[9] else 0,
            }

            conn.close()
            self.send_json(result)
        except Exception as e:
            self.send_json({'error': str(e)}, 500)

    def serve_positions(self):
        """Get position data for visualization"""
        try:
            conn = duckdb.connect(DB_PATH, read_only=True)

            # Get recent closed positions
            positions = conn.execute("""
                SELECT
                    mint,
                    pnl_percent,
                    pnl_sol,
                    hold_duration_secs,
                    holder_count_entry,
                    holder_count_exit,
                    exit_reason
                FROM positions
                WHERE exit_time_micros IS NOT NULL
                ORDER BY exit_time_micros DESC
                LIMIT 50
            """).fetchall()

            result = [{
                'mint': row[0][:8] + '...',
                'pnl_pct': float(row[1]) if row[1] else 0,
                'pnl_sol': float(row[2]) if row[2] else 0,
                'hold_secs': int(row[3]) if row[3] else 0,
                'entry_holders': int(row[4]) if row[4] else 0,
                'exit_holders': int(row[5]) if row[5] else 0,
                'exit_reason': row[6] or 'unknown'
            } for row in positions]

            conn.close()
            self.send_json(result)
        except Exception as e:
            self.send_json({'error': str(e)}, 500)

    def serve_recent_trades(self):
        """Get recent trades"""
        try:
            conn = duckdb.connect(DB_PATH, read_only=True)

            trades = conn.execute("""
                SELECT
                    trade_type,
                    mint,
                    price,
                    sol_amount,
                    to_timestamp(timestamp_micros / 1000000.0) as trade_time
                FROM trades
                ORDER BY timestamp_micros DESC
                LIMIT 20
            """).fetchall()

            result = [{
                'type': row[0],
                'mint': row[1][:8] + '...',
                'price': float(row[2]) if row[2] else 0,
                'sol': float(row[3]) if row[3] else 0,
                'time': str(row[4])
            } for row in trades]

            conn.close()
            self.send_json(result)
        except Exception as e:
            self.send_json({'error': str(e)}, 500)

    def send_json(self, data, status=200):
        """Send JSON response"""
        self.send_response(status)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Access-Control-Allow-Origin', '*')
        self.end_headers()
        self.wfile.write(json.dumps(data).encode())

def run(port=8080):
    os.chdir(os.path.dirname(os.path.abspath(__file__)))
    server = HTTPServer(('localhost', port), DashboardHandler)
    print(f"📊 Dashboard server running at http://localhost:{port}")
    print(f"   Open index.html in your browser")
    server.serve_forever()

if __name__ == '__main__':
    run()
