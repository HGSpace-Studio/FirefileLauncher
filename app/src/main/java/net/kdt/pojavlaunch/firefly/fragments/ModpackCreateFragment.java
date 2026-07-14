package net.kdt.pojavlaunch.firefly.fragments;

import android.app.Activity;
import android.os.Bundle;
import android.view.View;

import androidx.annotation.NonNull;
import androidx.annotation.Nullable;
import androidx.fragment.app.Fragment;

import net.kdt.pojavlaunch.firefly.LauncherActivity;
import net.kdt.pojavlaunch.firefly.R;
import net.kdt.pojavlaunch.firefly.Tools;

public class ModpackCreateFragment extends Fragment {
    public static final String TAG = "ModpackCreateFragment";
    public ModpackCreateFragment() {
        super(R.layout.fragment_create_modpack_profile);
    }

    @Override
    public void onViewCreated(@NonNull View view, @Nullable Bundle savedInstanceState) {
        view.findViewById(R.id.button_browse_modpacks).setOnClickListener(v -> {
            Tools.swapFragment(requireActivity(), SearchModFragment.class, SearchModFragment.TAG, null);
        });
        view.findViewById(R.id.button_import_modpack).setOnClickListener(v -> {
            Activity launcheractivity = requireActivity();
            if (!(launcheractivity instanceof LauncherActivity))
                    throw new IllegalStateException("Cannot import modpack without LauncherActivity");
            ((LauncherActivity) launcheractivity).modpackImportLauncher.launch(null);
        });
    }
}
