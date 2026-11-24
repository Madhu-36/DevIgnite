import React from 'react';
import { Search, Filter } from 'lucide-react';

interface SearchBarProps {
  searchQuery: string;
  onSearchChange: (query: string) => void;
  selectedTool: string;
  onToolSelect: (tool: string) => void;
}

const TOOLS = ['all', 'node', 'python', 'go', 'rust'];

export const SearchBar: React.FC<SearchBarProps> = ({
  searchQuery,
  onSearchChange,
  selectedTool,
  onToolSelect,
}) => {
  return (
    <div className="flex flex-col sm:flex-row gap-3 items-center justify-between">
      <div className="relative w-full sm:w-64">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" size={16} />
        <input
          type="text"
          value={searchQuery}
          onChange={(e) => onSearchChange(e.target.value)}
          placeholder="Search runtimes..."
          className="w-full bg-gray-800 border border-gray-700 text-white text-xs rounded-lg pl-9 pr-3 py-2 focus:outline-none focus:border-blue-500 transition-colors"
        />
      </div>

      <div className="flex items-center space-x-1 bg-gray-800 p-1 rounded-lg border border-gray-700 w-full sm:w-auto overflow-x-auto">
        <Filter size={14} className="text-gray-400 ml-2 mr-1 hidden sm:block shrink-0" />
        {TOOLS.map((tool) => (
          <button
            key={tool}
            onClick={() => onToolSelect(tool)}
            className={`px-3 py-1 text-xs rounded-md capitalize transition-all font-medium ${
              selectedTool === tool
                ? 'bg-blue-600 text-white shadow-sm'
                : 'text-gray-400 hover:text-white hover:bg-gray-700/50'
            }`}
          >
            {tool}
          </button>
        ))}
      </div>
    </div>
  );
};
