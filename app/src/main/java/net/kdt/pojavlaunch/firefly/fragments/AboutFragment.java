package net.kdt.pojavlaunch.firefly.fragments;

import android.os.Bundle;
import android.view.View;
import android.widget.Button;

import androidx.annotation.NonNull;
import androidx.annotation.Nullable;
import androidx.fragment.app.Fragment;

import net.kdt.pojavlaunch.firefly.R;
import net.kdt.pojavlaunch.firefly.Tools;

public class AboutFragment extends Fragment {
    public static final String TAG = "ABOUT_FRAGMENT";
    private static final String GITHUB_URL_PLT = "https://github.com/PojavLauncherTeam";
    private static final String GITHUB_URL_HGSPACE = "https://github.com/HGSpace-Studio";

    public AboutFragment() {
        super(R.layout.fragment_about);
    }

    @Override
    public void onViewCreated(@NonNull View view, @Nullable Bundle savedInstanceState) {
        Button mContributorButton1 = view.findViewById(R.id.contributor_pojavteam);
        Button mContributorButton2 = view.findViewById(R.id.contributor_happygray);
        Button mContributorButton3 = view.findViewById(R.id.contributor_hellochl);

        mContributorButton1.setOnClickListener(v -> Tools.openURL(requireActivity(), GITHUB_URL_PLT));
        mContributorButton2.setOnClickListener(v -> Tools.openURL(requireActivity(), GITHUB_URL_HGSPACE));
        mContributorButton3.setOnClickListener(v -> Tools.openURL(requireActivity(), GITHUB_URL_HGSPACE));
    }
}
